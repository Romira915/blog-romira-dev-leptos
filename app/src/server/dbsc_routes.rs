use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use tower_sessions::Session;

use crate::server::auth::get_current_user;
use crate::server::contexts::AppState;
use crate::server::services::dbsc::{
    DBSC_CHALLENGE_NONCES_KEY, DBSC_PUBLIC_KEY_KEY, DBSC_REGISTRATION_NONCE_KEY,
    DBSC_SESSION_ID_KEY, DbscService,
};

/// DBSC Registration endpoint — `POST /auth/dbsc/start`
async fn dbsc_registration(
    State(app_state): State<AppState>,
    session: Session,
    body: String,
) -> impl IntoResponse {
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

    // 3. Service: JWT検証・nonce照合・セッションID生成・Cookie構築
    let completion = match app_state
        .dbsc_service()
        .complete_registration(&body, &stored_nonce)
    {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!("DBSC registration failed: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 4. Remove registration nonce (one-time use)
    let _ = session.remove::<String>(DBSC_REGISTRATION_NONCE_KEY).await;

    // 5. Store results in session
    if let Err(e) = session
        .insert(DBSC_SESSION_ID_KEY, &completion.session_id)
        .await
    {
        tracing::error!("Failed to store DBSC session ID: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if let Err(e) = session
        .insert(DBSC_PUBLIC_KEY_KEY, &completion.public_key_jwk)
        .await
    {
        tracing::error!("Failed to store DBSC public key: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 6. Build HTTP response
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&completion.set_cookie_header) {
        headers.insert(axum::http::header::SET_COOKIE, v);
    }

    (headers, Json(completion.session_config)).into_response()
}

/// DBSC Refresh endpoint — `POST /auth/dbsc/refresh`
///
/// Two-phase challenge-response:
/// - Phase 1 (no `Secure-Session-Response`): Issue challenge with 403
/// - Phase 2 (with `Secure-Session-Response`): Verify proof and update cookie
async fn dbsc_refresh(
    State(app_state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Extract HTTP inputs
    let dbsc_session_id = headers
        .get("Sec-Secure-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(dbsc_session_id) = dbsc_session_id else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let jwt_proof = headers
        .get("Secure-Session-Response")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 2. Read session data
    let stored_session_id: Option<String> = session.get(DBSC_SESSION_ID_KEY).await.unwrap_or(None);

    if let Some(jwt_proof) = jwt_proof {
        // Phase 2: Verify proof and update cookie
        return handle_refresh_phase2(
            &session,
            app_state.dbsc_service(),
            &dbsc_session_id,
            stored_session_id.as_deref(),
            &jwt_proof,
        )
        .await;
    }

    // Phase 1: Issue challenge
    handle_refresh_phase1(&session, &dbsc_session_id, stored_session_id.as_deref()).await
}

async fn handle_refresh_phase1(
    session: &Session,
    dbsc_session_id: &str,
    stored_session_id: Option<&str>,
) -> axum::response::Response {
    // 1. Read current nonces from session
    let current_nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    // 2. Service: セッションID照合・nonce生成・リスト更新・チャレンジヘッダー構築
    let challenge =
        match DbscService::issue_challenge(dbsc_session_id, stored_session_id, current_nonces) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("DBSC challenge issue failed: {}", e);
                return StatusCode::NOT_FOUND.into_response();
            }
        };

    // 3. Store updated nonces in session
    if let Err(e) = session
        .insert(DBSC_CHALLENGE_NONCES_KEY, &challenge.updated_nonces)
        .await
    {
        tracing::error!("Failed to store DBSC challenge nonces: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 4. Build HTTP response
    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&challenge.challenge_header) {
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
    stored_session_id: Option<&str>,
    jwt_proof: &str,
) -> axum::response::Response {
    // 1. Read session data
    let public_key_jwk: Option<String> = session.get(DBSC_PUBLIC_KEY_KEY).await.unwrap_or(None);
    let nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    // 2. Service: セッションID照合・公開鍵/nonces検証・JWT検証・nonce消費・新Cookie発行
    let refresh = match dbsc_service.complete_refresh(
        jwt_proof,
        dbsc_session_id,
        stored_session_id,
        public_key_jwk.as_deref(),
        nonces,
    ) {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!("DBSC refresh failed for session {}: {}", dbsc_session_id, e);
            return StatusCode::FORBIDDEN.into_response();
        }
    };

    // 3. Store updated nonces in session
    if let Err(e) = session
        .insert(DBSC_CHALLENGE_NONCES_KEY, &refresh.updated_nonces)
        .await
    {
        tracing::error!("Failed to update DBSC challenge nonces: {}", e);
    }

    // 4. Build HTTP response
    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&refresh.set_cookie_header) {
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
