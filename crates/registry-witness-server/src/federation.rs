// SPDX-License-Identifier: Apache-2.0
//! Federated Registry Witness delegated evaluation routes.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::{
    env,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use axum::body::{to_bytes, Body};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::post;
use axum::{Extension, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use jsonwebtoken::{decode_header, Algorithm};
use registry_platform_crypto::{pairwise_subject_ref_hash, sign, PrivateJwk};
use registry_platform_httputil::FetchUrlPolicy;
use registry_platform_oidc::{
    JwksFetcher, JwksFetcherConfig, TokenVerifier, TokenVerifierConfig, VerifiedToken,
};
use registry_witness_core::{
    AccessMode, BoundedCorrelationId, ConfigMetadata, EvaluateRequest, EvidenceError,
    EvidencePrincipal, FederationConfig, FederationEvaluationProfileConfig, FederationPeerConfig,
    SourceCapability, SubjectRequest, FEDERATION_PROTOCOL_V0_1, FEDERATION_REQUEST_JWT_TYP,
    FEDERATION_RESPONSE_JWT_TYP, FORMAT_CLAIM_RESULT_JSON,
};
use serde_json::{json, Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::{
    api::{
        evidence_claim_hash, evidence_detail, evidence_status, evidence_title,
        RegistryWitnessApiState,
    },
    RegistryWitnessRuntime,
};

pub fn federation_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/federation/v1/evaluations", post(federated_evaluate))
}

async fn federated_evaluate(
    headers: HeaderMap,
    state: Option<Extension<Arc<RegistryWitnessApiState>>>,
    body: Body,
) -> Response {
    let started = Instant::now();
    let Some(Extension(state)) = state else {
        return federation_problem_response(FederationProblem::server_disabled());
    };
    let Some(runtime) = state.federation_runtime.as_ref().cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let outcome =
        handle_federated_evaluate(&headers, Arc::clone(&state), Arc::clone(&runtime), body).await;
    let (mut response, audit) = match outcome {
        Ok(outcome) => outcome.into_response(&runtime.response_signer),
        Err(problem) => {
            apply_denial_latency(
                started,
                state.federation.response_shaping.minimum_denial_latency_ms,
            )
            .await;
            let audit = FederationAuditOutcome::denied(&problem);
            (federation_problem_response(problem), audit)
        }
    };
    if let Some(audit_pipeline) = runtime.audit.as_ref() {
        let event = federation_audit_event(&headers, &response, audit, Some(audit_pipeline));
        if let Err(error) = audit_pipeline.emit(&event).await {
            response = crate::standalone::audit_error_response(error);
        }
    }
    response
}

async fn handle_federated_evaluate(
    headers: &HeaderMap,
    state: Arc<RegistryWitnessApiState>,
    runtime: Arc<FederationRuntimeState>,
    body: Body,
) -> Result<FederationSignedOutcome, FederationProblem> {
    state
        .enabled_evidence()
        .map_err(|_| FederationProblem::server_disabled())?;
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or_default().trim())
        != Some("application/jwt")
    {
        return Err(FederationProblem::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported-media-type",
            "Federation request content type must be application/jwt",
            "federation.unsupported_media_type",
        ));
    }
    let body = to_bytes(body, state.federation.inbound_body_limit_bytes)
        .await
        .map_err(|_| {
            FederationProblem::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload-too-large",
                "Federation request is too large",
                "federation.payload_too_large",
            )
        })?;
    let token = std::str::from_utf8(&body)
        .map(str::trim)
        .map_err(|_| FederationProblem::invalid_request("request body must be a compact JWS"))?;
    if token.split('.').count() != 3 {
        return Err(FederationProblem::invalid_request(
            "request body must be a compact JWS",
        ));
    }
    let header = decode_header(token).map_err(|_| FederationProblem::invalid_token())?;
    if header.alg != Algorithm::EdDSA {
        return Err(FederationProblem::invalid_token());
    }
    if header.typ.as_deref() != Some(FEDERATION_REQUEST_JWT_TYP) {
        return Err(FederationProblem::invalid_token());
    }
    let kid = header
        .kid
        .as_deref()
        .ok_or_else(FederationProblem::invalid_token)?;
    if state
        .federation
        .emergency_denylist
        .kids
        .iter()
        .any(|denied| denied == kid)
    {
        return Err(FederationProblem::forbidden("signing key is denied"));
    }
    let unverified = decode_unverified_jwt_payload(token)?;
    let issuer = string_claim(&unverified, "iss")
        .ok_or_else(FederationProblem::invalid_token)?
        .to_string();
    let peer = runtime
        .peers_by_issuer
        .get(&issuer)
        .ok_or_else(FederationProblem::invalid_token)?;
    if state
        .federation
        .emergency_denylist
        .node_ids
        .iter()
        .any(|denied| denied == &peer.config.node_id)
    {
        return Err(FederationProblem::forbidden("peer node is denied"));
    }
    let verified = peer
        .verifier
        .verify(token)
        .await
        .map_err(|_| FederationProblem::invalid_token())?;
    validate_federation_claims(&state.federation, &peer.config, &verified)?;
    let request_jti = string_extra(&verified, "jti")
        .ok_or_else(FederationProblem::invalid_token)?
        .to_string();
    let exp = verified
        .claims
        .exp
        .ok_or_else(FederationProblem::invalid_token)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if !runtime
        .replay
        .insert_once(
            &peer.config.issuer,
            &request_jti,
            exp,
            state.federation.clock_leeway_seconds,
            now,
            state.federation.replay.max_entries,
        )
        .await
    {
        return Err(FederationProblem::new(
            StatusCode::CONFLICT,
            "replay",
            "Federation request replay detected",
            "federation.replay",
        ));
    }
    let protocol = string_extra(&verified, "protocol")
        .ok_or_else(FederationProblem::invalid_request_owned)?
        .to_string();
    let profile_id = string_extra(&verified, "profile")
        .ok_or_else(FederationProblem::invalid_request_owned)?
        .to_string();
    let purpose = string_extra(&verified, "purpose")
        .ok_or_else(FederationProblem::invalid_request_owned)?
        .to_string();
    let profile = state
        .federation
        .evaluation_profiles
        .iter()
        .find(|candidate| candidate.id == profile_id)
        .ok_or_else(|| FederationProblem::forbidden("profile is not allowed"))?;
    let subject = request_subject(&verified, profile)?;
    let principal = EvidencePrincipal {
        principal_id: peer.config.node_id.clone(),
        scopes: peer.config.source_scopes.clone(),
        access_mode: AccessMode::MachineClient,
        verified_claims: None,
    };
    let source_capability = SourceCapability::Machine {
        scopes: peer
            .config
            .source_scopes
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
    };
    let request = EvaluateRequest {
        subject: subject.clone(),
        claims: vec![profile.claim_id.clone()],
        disclosure: Some("predicate".to_string()),
        format: Some(FORMAT_CLAIM_RESULT_JSON.to_string()),
        purpose: Some(purpose.clone()),
    };
    let subject_hash = pairwise_subject_ref_hash(
        runtime.pairwise_subject_hash_secret.as_slice(),
        &peer.config.node_id,
        &state.federation.node_id,
        &profile.id,
        subject.id_type.as_deref().unwrap_or(""),
        &subject.id,
    )
    .map_err(|_| FederationProblem::server_error("failed to hash subject reference"))?;
    let runtime_eval = RegistryWitnessRuntime::new_with_self_attestation_rate_keys(Arc::clone(
        &state.self_attestation_rate_keys,
    ));
    let results = runtime_eval
        .evaluate_with_source_capability(
            Arc::clone(&state.evidence),
            Arc::clone(&state.source),
            &state.store,
            &principal,
            source_capability,
            request,
            None,
            None,
            None,
        )
        .await
        .map_err(FederationProblem::from_evidence_error)?;
    if source_observation_is_stale(profile, &results) {
        return Ok(FederationSignedOutcome::evaluation_error(
            &state.federation,
            &peer.config,
            &protocol,
            profile,
            &purpose,
            &request_jti,
            subject_hash,
            "urn:registry-witness:problem:federation:stale-source-observation",
            "Source observation is stale",
        ));
    }
    Ok(FederationSignedOutcome::success(
        &state.federation,
        &peer.config,
        &protocol,
        profile,
        &purpose,
        &request_jti,
        subject.id_type.as_deref().unwrap_or(""),
        subject_hash,
        &results,
    ))
}

impl FederationReplayStore {
    async fn insert_once(
        &self,
        issuer: &str,
        jti: &str,
        exp: i64,
        clock_leeway_seconds: u64,
        now: i64,
        max_entries: usize,
    ) -> bool {
        let mut entries = self.entries.lock().await;
        let before_expiry_retain = entries.len();
        entries.retain(|_, entry| entry.expires_at >= now);
        self.evictions
            .fetch_add(before_expiry_retain - entries.len(), Ordering::Relaxed);
        let key = format!("{issuer}:{jti}");
        if entries.contains_key(&key) {
            return false;
        }
        while entries.len() >= max_entries {
            let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.inserted_sequence)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            entries.remove(&oldest);
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
        entries.insert(
            key,
            FederationReplayEntry {
                expires_at: exp.saturating_add(clock_leeway_seconds as i64),
                inserted_sequence: self.next_sequence.fetch_add(1, Ordering::Relaxed),
            },
        );
        true
    }
}

#[derive(Debug)]
struct FederationProblem {
    status: StatusCode,
    problem_type: String,
    title: String,
    detail: String,
    code: String,
}

impl FederationProblem {
    fn new(status: StatusCode, suffix: &str, title: &str, code: &str) -> Self {
        Self {
            status,
            problem_type: format!("urn:registry-witness:problem:federation:{suffix}"),
            title: title.to_string(),
            detail: title.to_ascii_lowercase(),
            code: code.to_string(),
        }
    }

    fn invalid_request(detail: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            problem_type: "urn:registry-witness:problem:federation:invalid-request".to_string(),
            title: "Invalid federation request".to_string(),
            detail: detail.to_string(),
            code: "federation.invalid_request".to_string(),
        }
    }

    fn invalid_request_owned() -> Self {
        Self::invalid_request("required federation claim is missing")
    }

    fn invalid_token() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "invalid-token",
            "Invalid federation token",
            "federation.invalid_token",
        )
    }

    fn forbidden(detail: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            problem_type: "urn:registry-witness:problem:federation:forbidden".to_string(),
            title: "Federation request forbidden".to_string(),
            detail: detail.to_string(),
            code: "federation.forbidden".to_string(),
        }
    }

    fn server_disabled() -> Self {
        Self::new(
            StatusCode::NOT_IMPLEMENTED,
            "disabled",
            "Federation is disabled",
            "federation.disabled",
        )
    }

    fn server_error(detail: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            problem_type: "urn:registry-witness:problem:federation:server-error".to_string(),
            title: "Federation server error".to_string(),
            detail: detail.to_string(),
            code: "federation.server_error".to_string(),
        }
    }

    fn from_evidence_error(error: EvidenceError) -> Self {
        let status = evidence_status(&error);
        Self {
            status,
            problem_type: format!("urn:registry-witness:problem:federation:{}", error.code()),
            title: evidence_title(&error).to_string(),
            detail: evidence_detail(&error).to_string(),
            code: error.audit_code().to_string(),
        }
    }
}

fn federation_problem_response(problem: FederationProblem) -> Response {
    let body = json!({
        "type": problem.problem_type,
        "title": problem.title,
        "status": problem.status.as_u16(),
        "detail": problem.detail,
        "code": problem.code,
        "instance": format!("urn:ulid:{}", Ulid::new()),
    });
    let mut response = (problem.status, Json(body)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

async fn apply_denial_latency(started: Instant, minimum_denial_latency_ms: u64) {
    let floor = Duration::from_millis(minimum_denial_latency_ms);
    let elapsed = started.elapsed();
    if elapsed < floor {
        tokio::time::sleep(floor - elapsed).await;
    }
}

#[derive(Debug)]
struct FederationSignedOutcome {
    claims: Value,
    audit: FederationAuditOutcome,
}

impl FederationSignedOutcome {
    #[allow(clippy::too_many_arguments)]
    fn success(
        federation: &FederationConfig,
        peer: &FederationPeerConfig,
        protocol: &str,
        profile: &FederationEvaluationProfileConfig,
        purpose: &str,
        request_jti: &str,
        subject_id_type: &str,
        subject_hash: String,
        results: &[registry_witness_core::ClaimResultView],
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let evaluation_id = results
            .first()
            .map(|result| format!("eval_{}", result.evaluation_id))
            .unwrap_or_else(|| format!("eval_{}", Ulid::new()));
        let mut claims = Map::new();
        for result in results {
            claims.insert(
                result.claim_id.clone(),
                json!({
                    "satisfied": result.satisfied,
                    "disclosure": result.disclosure,
                    "value": result.value,
                }),
            );
        }
        let source_observed_at = results.first().map(|result| result.issued_at.clone());
        let subject_ref_hash = subject_hash;
        let body = federation_base_response_claims(
            federation,
            peer,
            protocol,
            &profile.id,
            request_jti,
            now,
            "result",
            json!({
                "evaluation_id": evaluation_id,
                "subject_ref": {
                    "hash": subject_ref_hash.clone(),
                    "id_type": subject_id_type,
                },
                "source_observed_at": source_observed_at,
                "claims": Value::Object(claims),
            }),
        );
        Self {
            claims: body,
            audit: FederationAuditOutcome {
                decision: "federated_evaluate".to_string(),
                verification_id: Some(evaluation_id),
                claim_ids: vec![profile.claim_id.clone()],
                error_code: None,
                peer_node_id: Some(peer.node_id.clone()),
                issuer: Some(peer.issuer.clone()),
                profile: Some(profile.id.clone()),
                purpose: Some(purpose.to_string()),
                request_jti: Some(request_jti.to_string()),
                subject_ref_hash: Some(subject_ref_hash),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluation_error(
        federation: &FederationConfig,
        peer: &FederationPeerConfig,
        protocol: &str,
        profile: &FederationEvaluationProfileConfig,
        purpose: &str,
        request_jti: &str,
        subject_hash: String,
        error_type: &str,
        title: &str,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let body = federation_base_response_claims(
            federation,
            peer,
            protocol,
            &profile.id,
            request_jti,
            now,
            "error",
            json!({
                "type": error_type,
                "title": title,
                "code": "federation.stale_source_observation",
            }),
        );
        Self {
            claims: body,
            audit: FederationAuditOutcome {
                decision: "federated_evaluate_error".to_string(),
                verification_id: None,
                claim_ids: vec![profile.claim_id.clone()],
                error_code: Some("federation.stale_source_observation".to_string()),
                peer_node_id: Some(peer.node_id.clone()),
                issuer: Some(peer.issuer.clone()),
                profile: Some(profile.id.clone()),
                purpose: Some(purpose.to_string()),
                request_jti: Some(request_jti.to_string()),
                subject_ref_hash: Some(subject_hash),
            },
        }
    }

    fn into_response(
        self,
        signer: &FederationResponseSigner,
    ) -> (Response, FederationAuditOutcome) {
        match sign_federation_response(signer, &self.claims) {
            Ok(jwt) => {
                let mut response = (StatusCode::OK, jwt).into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/jwt"),
                );
                (response, self.audit)
            }
            Err(problem) => {
                let audit = FederationAuditOutcome::denied(&problem);
                (federation_problem_response(problem), audit)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn federation_base_response_claims(
    federation: &FederationConfig,
    peer: &FederationPeerConfig,
    protocol: &str,
    profile_id: &str,
    request_jti: &str,
    now: i64,
    body_field: &str,
    result: Value,
) -> Value {
    let mut claims = Map::from_iter([
        ("iss".to_string(), json!(federation.issuer)),
        ("sub".to_string(), json!(federation.node_id)),
        ("aud".to_string(), json!(peer.node_id)),
        ("iat".to_string(), json!(now)),
        ("nbf".to_string(), json!(now)),
        ("exp".to_string(), json!(now + 300)),
        ("jti".to_string(), json!(Ulid::new().to_string())),
        ("request_jti".to_string(), json!(request_jti)),
        ("protocol".to_string(), json!(protocol)),
        ("action".to_string(), json!("evaluate")),
        ("profile".to_string(), json!(profile_id)),
    ]);
    claims.insert(body_field.to_string(), result);
    Value::Object(claims)
}

fn sign_federation_response(
    signer: &FederationResponseSigner,
    claims: &Value,
) -> Result<String, FederationProblem> {
    let header = json!({
        "alg": "EdDSA",
        "typ": FEDERATION_RESPONSE_JWT_TYP,
        "kid": signer.kid,
    });
    let signing_input = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).map_err(|_| {
            FederationProblem::server_error("failed to encode response header")
        })?),
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).map_err(|_| {
            FederationProblem::server_error("failed to encode response claims")
        })?)
    );
    let signature = sign(signing_input.as_bytes(), &signer.key)
        .map_err(|_| FederationProblem::server_error("failed to sign response"))?;
    Ok(format!(
        "{}.{}",
        signing_input,
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

#[derive(Debug)]
struct FederationAuditOutcome {
    decision: String,
    verification_id: Option<String>,
    claim_ids: Vec<String>,
    error_code: Option<String>,
    peer_node_id: Option<String>,
    issuer: Option<String>,
    profile: Option<String>,
    purpose: Option<String>,
    request_jti: Option<String>,
    subject_ref_hash: Option<String>,
}

impl FederationAuditOutcome {
    fn denied(problem: &FederationProblem) -> Self {
        Self {
            decision: "federated_evaluate_denied".to_string(),
            verification_id: None,
            claim_ids: Vec::new(),
            error_code: Some(problem.code.clone()),
            peer_node_id: None,
            issuer: None,
            profile: None,
            purpose: None,
            request_jti: None,
            subject_ref_hash: None,
        }
    }
}

fn federation_audit_event(
    headers: &HeaderMap,
    response: &Response,
    audit: FederationAuditOutcome,
    audit_pipeline: Option<&crate::standalone::AuditPipeline>,
) -> registry_witness_core::EvidenceAuditEvent {
    let occurred_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let federation_peer_id_hash = audit.peer_node_id.as_deref().and_then(|peer_node_id| {
        audit_pipeline.map(|pipeline| pipeline.hash_principal(peer_node_id))
    });
    registry_witness_core::EvidenceAuditEvent {
        event_id: Ulid::new().to_string(),
        occurred_at,
        principal_id_hash: None,
        decision: audit.decision,
        method: "POST".to_string(),
        path: "/federation/v1/evaluations".to_string(),
        status: response.status().as_u16(),
        verification_id: audit.verification_id,
        claim_hash: (!audit.claim_ids.is_empty()).then(|| evidence_claim_hash(&audit.claim_ids)),
        row_count: response.status().is_success().then_some(1),
        error_code: audit.error_code,
        access_mode: Some(AccessMode::MachineClient),
        federation_peer_id_hash,
        federation_issuer: audit.issuer,
        federation_profile: audit.profile,
        federation_purpose: audit.purpose,
        federation_request_jti: audit.request_jti,
        federation_subject_ref_hash: audit.subject_ref_hash,
        denial_code: None,
        token_claim_name: None,
        correlation_id: headers
            .get("x-request-id")
            .or_else(|| headers.get("x-correlation-id"))
            .and_then(|value| value.to_str().ok())
            .and_then(|value| BoundedCorrelationId::new(value.to_string()).ok()),
        credential_profile: None,
        protocol: ConfigMetadata::new(FEDERATION_PROTOCOL_V0_1).ok(),
        credential_configuration_id: None,
        holder_binding_mode: None,
        rate_limit_bucket: None,
        policy_version: None,
        policy_hash: None,
    }
}

fn validate_federation_claims(
    federation: &FederationConfig,
    peer: &FederationPeerConfig,
    verified: &VerifiedToken,
) -> Result<(), FederationProblem> {
    if verified.claims.sub.as_deref() != Some(peer.node_id.as_str()) {
        return Err(FederationProblem::invalid_token());
    }
    let Some(iat) = verified.claims.iat else {
        return Err(FederationProblem::invalid_token());
    };
    let Some(nbf) = verified.claims.nbf else {
        return Err(FederationProblem::invalid_token());
    };
    let Some(exp) = verified.claims.exp else {
        return Err(FederationProblem::invalid_token());
    };
    if nbf < iat.saturating_sub(federation.clock_leeway_seconds as i64) {
        return Err(FederationProblem::invalid_token());
    }
    if exp - iat > federation.max_request_lifetime_seconds as i64 {
        return Err(FederationProblem::invalid_token());
    }
    let jti = string_extra(verified, "jti").ok_or_else(FederationProblem::invalid_token)?;
    if Ulid::from_string(jti).is_err() {
        return Err(FederationProblem::invalid_token());
    }
    let protocol =
        string_extra(verified, "protocol").ok_or_else(FederationProblem::invalid_request_owned)?;
    if protocol != FEDERATION_PROTOCOL_V0_1
        || !peer
            .allowed_protocol_versions
            .iter()
            .any(|allowed| allowed == protocol)
    {
        return Err(FederationProblem::forbidden("protocol is not allowed"));
    }
    if string_extra(verified, "action") != Some("evaluate") {
        return Err(FederationProblem::invalid_request(
            "action must be evaluate",
        ));
    }
    let profile =
        string_extra(verified, "profile").ok_or_else(FederationProblem::invalid_request_owned)?;
    if !peer
        .allowed_profiles
        .iter()
        .any(|allowed| allowed == profile)
    {
        return Err(FederationProblem::forbidden("profile is not allowed"));
    }
    let purpose =
        string_extra(verified, "purpose").ok_or_else(FederationProblem::invalid_request_owned)?;
    if !peer
        .allowed_purposes
        .iter()
        .any(|allowed| allowed == purpose)
    {
        return Err(FederationProblem::forbidden("purpose is not allowed"));
    }
    Ok(())
}

fn request_subject(
    verified: &VerifiedToken,
    profile: &FederationEvaluationProfileConfig,
) -> Result<SubjectRequest, FederationProblem> {
    let request = verified
        .claims
        .extra
        .get("request")
        .and_then(Value::as_object)
        .ok_or_else(|| FederationProblem::invalid_request("request object is required"))?;
    let subject = request
        .get("subject")
        .and_then(Value::as_object)
        .ok_or_else(|| FederationProblem::invalid_request("request.subject is required"))?;
    let id = subject
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| FederationProblem::invalid_request("request.subject.id is required"))?;
    let id_type = subject
        .get("id_type")
        .and_then(Value::as_str)
        .ok_or_else(|| FederationProblem::invalid_request("request.subject.id_type is required"))?;
    if id_type != profile.subject_id_type {
        return Err(FederationProblem::forbidden(
            "subject id type is not allowed",
        ));
    }
    let requested_claims = request
        .get("claims")
        .and_then(Value::as_array)
        .ok_or_else(|| FederationProblem::invalid_request("request.claims is required"))?;
    if requested_claims.len() != 1
        || requested_claims.first().and_then(Value::as_str) != Some(profile.claim_id.as_str())
    {
        return Err(FederationProblem::forbidden(
            "request claims do not match profile",
        ));
    }
    Ok(SubjectRequest {
        id: id.to_string(),
        id_type: Some(id_type.to_string()),
    })
}

fn source_observation_is_stale(
    profile: &FederationEvaluationProfileConfig,
    results: &[registry_witness_core::ClaimResultView],
) -> bool {
    let Some(max_age) = profile.max_source_observed_age_seconds else {
        return false;
    };
    if max_age == 0 {
        return true;
    }
    let Some(observed_at) = results
        .first()
        .and_then(|result| OffsetDateTime::parse(&result.issued_at, &Rfc3339).ok())
    else {
        return true;
    };
    let age = OffsetDateTime::now_utc() - observed_at;
    age > time::Duration::seconds(max_age as i64)
}

fn string_extra<'a>(verified: &'a VerifiedToken, claim: &str) -> Option<&'a str> {
    verified.claims.extra.get(claim).and_then(Value::as_str)
}

fn string_claim<'a>(claims: &'a Value, claim: &str) -> Option<&'a str> {
    claims.get(claim).and_then(Value::as_str)
}

fn decode_unverified_jwt_payload(token: &str) -> Result<Value, FederationProblem> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(FederationProblem::invalid_token)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| FederationProblem::invalid_token())?;
    serde_json::from_slice(&bytes).map_err(|_| FederationProblem::invalid_token())
}

#[derive(Clone)]
pub(crate) struct FederationRuntimeState {
    response_signer: FederationResponseSigner,
    pairwise_subject_hash_secret: Arc<Vec<u8>>,
    peers_by_issuer: Arc<HashMap<String, FederationResolvedPeer>>,
    replay: Arc<FederationReplayStore>,
    audit: Option<crate::standalone::AuditPipeline>,
}

#[derive(Clone)]
struct FederationResponseSigner {
    kid: String,
    key: PrivateJwk,
}

#[derive(Clone)]
struct FederationResolvedPeer {
    config: FederationPeerConfig,
    verifier: Arc<TokenVerifier>,
}

#[derive(Default)]
struct FederationReplayStore {
    entries: Mutex<BTreeMap<String, FederationReplayEntry>>,
    next_sequence: AtomicU64,
    evictions: AtomicUsize,
}

#[derive(Debug, Clone, Copy)]
struct FederationReplayEntry {
    expires_at: i64,
    inserted_sequence: u64,
}

impl FederationRuntimeState {
    pub(crate) fn from_config(
        config: &FederationConfig,
        audit: Option<crate::standalone::AuditPipeline>,
    ) -> Result<Self, crate::standalone::StandaloneServerError> {
        let signing_key = env::var(&config.signing.key_env)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                crate::standalone::StandaloneServerError::MissingFederationSecretEnv(
                    config.signing.key_env.clone(),
                )
            })?;
        let key = PrivateJwk::parse(&signing_key).map_err(|error| {
            crate::standalone::StandaloneServerError::InvalidFederationSigningKeyEnv(
                config.signing.key_env.clone(),
                error.to_string(),
            )
        })?;
        let pairwise_subject_hash_secret = env::var(&config.pairwise_subject_hash.secret_env)
            .ok()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                crate::standalone::StandaloneServerError::MissingFederationSecretEnv(
                    config.pairwise_subject_hash.secret_env.clone(),
                )
            })?
            .into_bytes();
        let mut peers_by_issuer = HashMap::new();
        for peer in &config.peers {
            let fetch_url_policy = if peer.allow_insecure_localhost {
                FetchUrlPolicy::dev()
            } else {
                FetchUrlPolicy::strict()
            };
            let fetcher = Arc::new(JwksFetcher::new_with_fetch_url_policy(
                peer.jwks_uri.clone(),
                JwksFetcherConfig::defaults(),
                fetch_url_policy,
            ));
            let verifier = Arc::new(TokenVerifier::new(
                TokenVerifierConfig {
                    issuer: peer.issuer.clone(),
                    audiences: vec![config.node_id.clone()],
                    allowed_algorithms: vec![Algorithm::EdDSA],
                    allowed_typ: vec![FEDERATION_REQUEST_JWT_TYP.to_string()],
                    scope_claim: "scope".to_string(),
                    scope_separator: ' ',
                    scope_map: None,
                    allowed_clients: Vec::new(),
                    leeway: Duration::from_secs(config.clock_leeway_seconds),
                },
                fetcher,
            ));
            peers_by_issuer.insert(
                peer.issuer.clone(),
                FederationResolvedPeer {
                    config: peer.clone(),
                    verifier,
                },
            );
        }
        Ok(Self {
            response_signer: FederationResponseSigner {
                kid: config.signing.kid.clone(),
                key,
            },
            pairwise_subject_hash_secret: Arc::new(pairwise_subject_hash_secret),
            peers_by_issuer: Arc::new(peers_by_issuer),
            replay: Arc::new(FederationReplayStore::default()),
            audit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn federation_replay_store_retains_jti_until_exp_plus_leeway() {
        let store = FederationReplayStore::default();

        assert!(
            store
                .insert_once("https://issuer.example", "01JTI", 100, 60, 100, 10)
                .await
        );
        assert!(
            !store
                .insert_once("https://issuer.example", "01JTI", 100, 60, 150, 10)
                .await
        );
        assert!(
            store
                .insert_once("https://issuer.example", "01JTI", 100, 60, 161, 10)
                .await
        );
        assert_eq!(store.evictions.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn federation_replay_store_evicts_oldest_inserted_entry_when_full() {
        let store = FederationReplayStore::default();

        assert!(store.insert_once("issuer", "a", 1000, 0, 0, 2).await);
        assert!(store.insert_once("issuer", "b", 1000, 0, 0, 2).await);
        assert!(store.insert_once("issuer", "c", 1000, 0, 0, 2).await);

        assert!(!store.insert_once("issuer", "b", 1000, 0, 0, 2).await);
        assert_eq!(store.evictions.load(Ordering::Relaxed), 1);
    }
}
