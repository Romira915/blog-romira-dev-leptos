use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use tower_sessions::Session;

use crate::server::contexts::AppState;
use crate::server::services::dbsc::{
    DBSC_CHALLENGE_NONCES_KEY, DBSC_PUBLIC_KEY_KEY, DBSC_REGISTRATION_NONCE_KEY,
    DBSC_SESSION_ID_KEY, DbscService,
};

/// Dump all DBSC-relevant request information for debugging.
fn dump_request(
    label: &str,
    headers: &HeaderMap,
    session_id: Option<&tower_sessions::session::Id>,
) {
    tracing::debug!("===== DBSC {} REQUEST DUMP =====", label);

    // All headers
    for (name, value) in headers.iter() {
        let val = value.to_str().unwrap_or("<non-utf8>");
        // Truncate long values (JWTs)
        if val.len() > 100 {
            tracing::debug!("  header: {}={:.100}...(len={})", name, val, val.len());
        } else {
            tracing::debug!("  header: {}={}", name, val);
        }
    }

    // All cookies (parsed)
    let cookies: Vec<String> = headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(|s| s.trim().to_string())
        .collect();
    if cookies.is_empty() {
        tracing::debug!("  cookies: NONE");
    } else {
        for c in &cookies {
            // Truncate long cookie values
            if c.len() > 80 {
                tracing::debug!("  cookie: {:.80}...(len={})", c, c.len());
            } else {
                tracing::debug!("  cookie: {}", c);
            }
        }
    }

    // Session
    tracing::debug!("  session_id: {:?}", session_id);
    tracing::debug!("===== END {} DUMP =====", label);
}

/// DBSC Registration endpoint — `POST /auth/dbsc/start`
///
/// Chrome sends the JWT proof in the `Secure-Session-Response` header (not in the body).
/// Session cookie IS available thanks to JS redirect resetting Chrome's initiator.
async fn dbsc_registration(
    State(app_state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> impl IntoResponse {
    dump_request("REGISTRATION", &headers, session.id().as_ref());

    // 1. Extract JWT from Secure-Session-Response header (strip sf-string quotes if present)
    let jwt_proof = match headers
        .get("Secure-Session-Response")
        .and_then(|v| v.to_str().ok())
    {
        Some(jwt) => {
            let trimmed = jwt.trim_matches('"').to_string();
            tracing::debug!("DBSC registration: JWT extracted (len={})", trimmed.len());
            trimmed
        }
        None => {
            tracing::warn!("DBSC registration: Secure-Session-Response header missing → 400");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 2. Get nonce from session
    let stored_nonce: Option<String> = session
        .get(DBSC_REGISTRATION_NONCE_KEY)
        .await
        .unwrap_or(None);
    let Some(stored_nonce) = stored_nonce else {
        tracing::warn!("DBSC registration: no registration nonce in session → 400");
        return StatusCode::BAD_REQUEST.into_response();
    };
    tracing::debug!("DBSC registration: stored_nonce={}", stored_nonce);

    // 3. Service: JWT検証・nonce照合・セッションID生成・Cookie構築
    let completion = match app_state
        .dbsc_service()
        .complete_registration(&jwt_proof, &stored_nonce)
    {
        Ok(result) => {
            tracing::info!(
                "DBSC registration: SUCCESS, session_id={}",
                result.session_id
            );
            result
        }
        Err(e) => {
            tracing::warn!("DBSC registration FAILED: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 3. Store DBSC data in session (session cookie is available thanks to JS redirect
    //    resetting the initiator, so SameSite=Lax cookies are included)
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
    // Remove registration nonce (one-time use)
    let _ = session.remove::<String>(DBSC_REGISTRATION_NONCE_KEY).await;
    tracing::debug!("DBSC registration: stored session_id and public_key in session");

    // 4. Build HTTP response
    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&completion.set_cookie_header) {
        tracing::debug!(
            "DBSC registration: Set-Cookie (dbsc): {}",
            completion.set_cookie_header
        );
        response_headers.append(axum::http::header::SET_COOKIE, v);
    }

    tracing::debug!(
        "DBSC registration: response session_config={}",
        completion.session_config
    );

    (response_headers, Json(completion.session_config)).into_response()
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
    dump_request("REFRESH", &headers, session.id().as_ref());

    // Log session contents
    let session_dbsc_id: Option<String> = session.get(DBSC_SESSION_ID_KEY).await.unwrap_or(None);
    let session_pubkey: Option<String> = session.get(DBSC_PUBLIC_KEY_KEY).await.unwrap_or(None);
    let session_nonces: Option<Vec<String>> =
        session.get(DBSC_CHALLENGE_NONCES_KEY).await.unwrap_or(None);
    tracing::debug!(
        "DBSC refresh: session state: dbsc_session_id={:?}, has_public_key={}, nonces={:?}",
        session_dbsc_id,
        session_pubkey.is_some(),
        session_nonces
    );

    // 1. Extract HTTP inputs (strip sf-string quotes if present)
    let dbsc_session_id = headers
        .get("Sec-Secure-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());

    tracing::debug!("DBSC refresh: Sec-Secure-Session-Id={:?}", dbsc_session_id);

    let Some(dbsc_session_id) = dbsc_session_id else {
        tracing::warn!("DBSC refresh: Sec-Secure-Session-Id missing → 400");
        return StatusCode::BAD_REQUEST.into_response();
    };

    let jwt_proof = headers
        .get("Secure-Session-Response")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());

    tracing::debug!(
        "DBSC refresh: Secure-Session-Response present={}, len={:?}",
        jwt_proof.is_some(),
        jwt_proof.as_ref().map(|j| j.len())
    );

    // 2. Read session data (DBSC data is written directly to session during registration)
    let stored_session_id = session_dbsc_id;

    tracing::debug!(
        "DBSC refresh: final stored_session_id={:?}, request_session_id={}",
        stored_session_id,
        dbsc_session_id
    );

    if let Some(jwt_proof) = jwt_proof {
        tracing::debug!("DBSC refresh: entering Phase 2 (proof verification)");
        return handle_refresh_phase2(
            &session,
            app_state.dbsc_service(),
            &dbsc_session_id,
            stored_session_id.as_deref(),
            &jwt_proof,
        )
        .await;
    }

    tracing::debug!("DBSC refresh: entering Phase 1 (challenge issue)");
    handle_refresh_phase1(&session, &dbsc_session_id, stored_session_id.as_deref()).await
}

async fn handle_refresh_phase1(
    session: &Session,
    dbsc_session_id: &str,
    stored_session_id: Option<&str>,
) -> axum::response::Response {
    let current_nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    tracing::debug!(
        "DBSC refresh phase1: current_nonces_count={}, dbsc_session_id={}, stored_session_id={:?}",
        current_nonces.len(),
        dbsc_session_id,
        stored_session_id
    );

    let challenge =
        match DbscService::issue_challenge(dbsc_session_id, stored_session_id, current_nonces) {
            Ok(result) => {
                tracing::debug!(
                    "DBSC refresh phase1: challenge issued, header={}",
                    result.challenge_header
                );
                result
            }
            Err(e) => {
                tracing::warn!("DBSC refresh phase1 FAILED: {} → 404", e);
                return StatusCode::NOT_FOUND.into_response();
            }
        };

    if let Err(e) = session
        .insert(DBSC_CHALLENGE_NONCES_KEY, &challenge.updated_nonces)
        .await
    {
        tracing::error!("Failed to store DBSC challenge nonces: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&challenge.challenge_header) {
        response_headers.insert("Secure-Session-Challenge", v);
    }
    response_headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    tracing::debug!("DBSC refresh phase1: returning 403 with challenge");
    (StatusCode::FORBIDDEN, response_headers).into_response()
}

async fn handle_refresh_phase2(
    session: &Session,
    dbsc_service: &DbscService,
    dbsc_session_id: &str,
    stored_session_id: Option<&str>,
    jwt_proof: &str,
) -> axum::response::Response {
    let public_key_jwk: Option<String> = session.get(DBSC_PUBLIC_KEY_KEY).await.unwrap_or(None);
    let nonces: Vec<String> = session
        .get(DBSC_CHALLENGE_NONCES_KEY)
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    tracing::debug!(
        "DBSC refresh phase2: has_public_key={}, nonces={:?}, stored_session_id={:?}, request_session_id={}, jwt_len={}",
        public_key_jwk.is_some(),
        nonces,
        stored_session_id,
        dbsc_session_id,
        jwt_proof.len()
    );

    let refresh = match dbsc_service.complete_refresh(
        jwt_proof,
        dbsc_session_id,
        stored_session_id,
        public_key_jwk.as_deref(),
        nonces,
    ) {
        Ok(result) => {
            tracing::debug!(
                "DBSC refresh phase2: SUCCESS, remaining_nonces={}",
                result.updated_nonces.len()
            );
            result
        }
        Err(e) => {
            tracing::warn!(
                "DBSC refresh phase2 FAILED for session {}: {} → 403",
                dbsc_session_id,
                e
            );
            return StatusCode::FORBIDDEN.into_response();
        }
    };

    if let Err(e) = session
        .insert(DBSC_CHALLENGE_NONCES_KEY, &refresh.updated_nonces)
        .await
    {
        tracing::error!("Failed to update DBSC challenge nonces: {}", e);
    }

    let mut response_headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&refresh.set_cookie_header) {
        tracing::debug!(
            "DBSC refresh phase2: Set-Cookie: {}",
            refresh.set_cookie_header
        );
        response_headers.insert(axum::http::header::SET_COOKIE, v);
    }
    // Include next challenge for Chrome to cache (skips Phase 1 on next Refresh)
    if let Ok(v) = HeaderValue::from_str(&refresh.challenge_header) {
        tracing::debug!(
            "DBSC refresh phase2: Secure-Session-Challenge: {}",
            refresh.challenge_header
        );
        response_headers.insert("Secure-Session-Challenge", v);
    }
    response_headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    tracing::debug!("DBSC refresh phase2: returning 200 OK");
    (StatusCode::OK, response_headers).into_response()
}

/// DBSC Registration initiation — `GET /auth/dbsc/registration`
///
/// Serves `Secure-Session-Registration` header with 303 redirect to /admin.
/// This endpoint is reached via JS redirect from auth_callback, which resets
/// Chrome's initiator from accounts.google.com to our domain.
/// As a result, the DBSC Registration POST includes SameSite=Lax session cookies.
async fn dbsc_registration_initiation(
    State(app_state): State<AppState>,
    session: Session,
) -> impl IntoResponse {
    let initiation = app_state.dbsc_service().initiate_registration();

    // Store nonce in session for verification in Registration POST
    if let Err(e) = session
        .insert(DBSC_REGISTRATION_NONCE_KEY, &initiation.nonce)
        .await
    {
        tracing::error!("Failed to store DBSC nonce: {}", e);
        return axum::response::Redirect::to("/admin").into_response();
    }
    tracing::debug!(
        "DBSC registration initiation: nonce={}, 303 /admin",
        initiation.nonce
    );

    let mut response = axum::response::Redirect::to("/admin").into_response();
    if let Ok(v) = HeaderValue::from_str(&initiation.header_value) {
        response
            .headers_mut()
            .insert("Secure-Session-Registration", v);
    }
    response
}

pub fn dbsc_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/auth/dbsc/registration",
            get(dbsc_registration_initiation).post(dbsc_registration),
        )
        .route("/auth/dbsc/refresh", post(dbsc_refresh))
}
