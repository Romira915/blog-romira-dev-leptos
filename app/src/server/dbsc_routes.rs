use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use tower_sessions::Session;

use crate::server::auth::get_current_user;
use crate::server::config::SERVER_CONFIG;
use crate::server::contexts::AppState;
use crate::server::services::dbsc::{
    DBSC_CHALLENGE_NONCES_KEY, DBSC_PUBLIC_KEY_KEY, DBSC_REGISTRATION_NONCE_KEY,
    DBSC_SESSION_ID_KEY, DbscService,
};

/// DBSC Registration endpoint — `POST /auth/dbsc/start`
async fn dbsc_registration(session: Session, body: String) -> impl IntoResponse {
    // 1. Verify user is authenticated
    if get_current_user(&session).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // 2. Get registration nonce from session
    let stored_nonce: Option<String> = session
        .get(DBSC_REGISTRATION_NONCE_KEY)
        .await
        .unwrap_or(None);
    let Some(stored_nonce) = stored_nonce else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    // 3-4. Verify JWT and extract public key
    let dbsc_service = DbscService::new(SERVER_CONFIG.app_url.clone());
    let (jti_nonce, public_key_jwk) = match dbsc_service.verify_registration_jwt(&body) {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!("DBSC registration JWT verification failed: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 5. Verify nonce matches
    if jti_nonce != stored_nonce {
        tracing::warn!("DBSC registration nonce mismatch");
        return StatusCode::FORBIDDEN.into_response();
    }

    // 6. Remove registration nonce (one-time use)
    let _ = session.remove::<String>(DBSC_REGISTRATION_NONCE_KEY).await;

    // 7. Generate DBSC session ID
    let dbsc_session_id = uuid::Uuid::now_v7().to_string();

    // 8. Store in session
    if let Err(e) = session.insert(DBSC_SESSION_ID_KEY, &dbsc_session_id).await {
        tracing::error!("Failed to store DBSC session ID: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if let Err(e) = session.insert(DBSC_PUBLIC_KEY_KEY, &public_key_jwk).await {
        tracing::error!("Failed to store DBSC public key: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 9. Set DBSC cookie
    let cookie_value = DbscService::generate_cookie_value();
    let set_cookie = DbscService::build_set_cookie_header(&cookie_value);

    // 10. Return session config JSON
    let session_config = dbsc_service.build_session_config(&dbsc_session_id);

    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&set_cookie) {
        headers.insert(axum::http::header::SET_COOKIE, v);
    }

    (headers, Json(session_config)).into_response()
}

/// DBSC Refresh endpoint — `POST /auth/dbsc/refresh`
///
/// Two-phase challenge-response:
/// - Phase 1 (no `Secure-Session-Response`): Issue challenge with 403
/// - Phase 2 (with `Secure-Session-Response`): Verify proof and update cookie
async fn dbsc_refresh(session: Session, headers: HeaderMap) -> impl IntoResponse {
    let dbsc_service = DbscService::new(SERVER_CONFIG.app_url.clone());

    // Get DBSC session ID from Sec-Secure-Session-Id header
    let dbsc_session_id = headers
        .get("Sec-Secure-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(dbsc_session_id) = dbsc_session_id else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    // Verify session has matching DBSC session ID
    let stored_session_id: Option<String> = session.get(DBSC_SESSION_ID_KEY).await.unwrap_or(None);
    if stored_session_id.as_ref() != Some(&dbsc_session_id) {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Check if this is Phase 2 (has Secure-Session-Response)
    let jwt_proof = headers
        .get("Secure-Session-Response")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(jwt_proof) = jwt_proof {
        // Phase 2: Verify proof and update cookie
        return handle_refresh_phase2(&session, &dbsc_service, &dbsc_session_id, &jwt_proof).await;
    }

    // Phase 1: Issue challenge
    handle_refresh_phase1(&session, &dbsc_session_id).await
}

async fn handle_refresh_phase1(
    session: &Session,
    dbsc_session_id: &str,
) -> axum::response::Response {
    let nonce = DbscService::generate_nonce();

    // Add nonce to challenge list in session
    let mut nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();
    DbscService::push_challenge_nonce(&mut nonces, nonce.clone());

    if let Err(e) = session.insert(DBSC_CHALLENGE_NONCES_KEY, &nonces).await {
        tracing::error!("Failed to store DBSC challenge nonces: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let challenge_header = DbscService::build_challenge_header(&nonce, dbsc_session_id);
    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&challenge_header) {
        response_headers.insert("Secure-Session-Challenge", v);
    }
    response_headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    (StatusCode::FORBIDDEN, response_headers).into_response()
}

async fn handle_refresh_phase2(
    session: &Session,
    dbsc_service: &DbscService,
    dbsc_session_id: &str,
    jwt_proof: &str,
) -> axum::response::Response {
    // Get public key from session
    let public_key_jwk: Option<String> = session.get(DBSC_PUBLIC_KEY_KEY).await.unwrap_or(None);
    let Some(public_key_jwk) = public_key_jwk else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Get challenge nonces from session
    let nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    if nonces.is_empty() {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Verify JWT proof
    let matched_nonce = match dbsc_service.verify_refresh_jwt(jwt_proof, &public_key_jwk, &nonces) {
        Ok(nonce) => nonce,
        Err(e) => {
            tracing::warn!(
                "DBSC refresh JWT verification failed for session {}: {}",
                dbsc_session_id,
                e
            );
            return StatusCode::FORBIDDEN.into_response();
        }
    };

    // Remove matched nonce from list
    let mut nonces = nonces;
    nonces.retain(|n| n != &matched_nonce);
    if let Err(e) = session.insert(DBSC_CHALLENGE_NONCES_KEY, &nonces).await {
        tracing::error!("Failed to update DBSC challenge nonces: {}", e);
    }

    // Issue new DBSC cookie
    let cookie_value = DbscService::generate_cookie_value();
    let set_cookie = DbscService::build_set_cookie_header(&cookie_value);

    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&set_cookie) {
        response_headers.insert(axum::http::header::SET_COOKIE, v);
    }
    response_headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    (StatusCode::OK, response_headers).into_response()
}

pub fn dbsc_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/dbsc/start", post(dbsc_registration))
        .route("/auth/dbsc/refresh", post(dbsc_refresh))
}
