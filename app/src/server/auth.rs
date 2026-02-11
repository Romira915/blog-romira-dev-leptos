use axum::extract::Query;
use axum::http::StatusCode;
use axum::{
    Router,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::get,
};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, StandardRevocableToken, TokenResponse, TokenUrl,
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::common::handlers::auth::AuthUser;
use crate::server::config::SERVER_CONFIG;
use crate::server::contexts::AppState;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

const SESSION_USER_KEY: &str = "user";
const SESSION_CSRF_KEY: &str = "oauth_csrf";

type GoogleOAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<oauth2::EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

/// Create OAuth2 client
fn create_oauth_client() -> GoogleOAuthClient {
    let redirect_url = format!("{}/auth/callback", SERVER_CONFIG.app_url);

    BasicClient::new(ClientId::new(SERVER_CONFIG.google_client_id.clone()))
        .set_client_secret(ClientSecret::new(
            SERVER_CONFIG.google_client_secret.clone(),
        ))
        .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

/// Start OAuth flow - redirect to Google
pub async fn auth_google(session: Session) -> impl IntoResponse {
    let client = create_oauth_client();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // Store CSRF token in session
    if let Err(e) = session
        .insert(SESSION_CSRF_KEY, csrf_token.secret().clone())
        .await
    {
        tracing::error!("Failed to store CSRF token: {}", e);
        return Redirect::to("/admin?error=session_error").into_response();
    }

    Redirect::to(auth_url.as_str()).into_response()
}

/// OAuth callback - exchange code for token and get user info
pub async fn auth_callback(
    session: Session,
    Query(query): Query<AuthCallbackQuery>,
) -> impl IntoResponse {
    // Verify CSRF token
    let stored_csrf: Option<String> = session.get(SESSION_CSRF_KEY).await.unwrap_or(None);
    if stored_csrf.as_ref() != Some(&query.state) {
        tracing::warn!("CSRF token mismatch");
        return Redirect::to("/admin?error=csrf_mismatch").into_response();
    }
    let _ = session.remove::<String>(SESSION_CSRF_KEY).await;

    let client = create_oauth_client();

    // Exchange code for token
    let http_client = oauth2::reqwest::Client::new();
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await;

    let token = match token_result {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to exchange code for token: {}", e);
            return Redirect::to("/admin?error=token_exchange_failed").into_response();
        }
    };

    // Get user info from Google
    let http_client = reqwest::Client::new();
    let userinfo_response = http_client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(token.access_token().secret())
        .send()
        .await;

    let userinfo: GoogleUserInfo = match userinfo_response {
        Ok(resp) => match resp.json().await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Failed to parse user info: {}", e);
                return Redirect::to("/admin?error=userinfo_parse_failed").into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to get user info: {}", e);
            return Redirect::to("/admin?error=userinfo_failed").into_response();
        }
    };

    // Check if email is in the admin allowlist
    if !is_admin_email(&userinfo.email) {
        tracing::warn!("Unauthorized login attempt: {}", userinfo.email);
        let _ = session.flush().await;
        return (
            StatusCode::FORBIDDEN,
            axum::response::Html(
                "<h1>403 Forbidden</h1><p>This account is not authorized to access the admin area.</p>",
            ),
        )
            .into_response();
    }

    // Store user in session
    let user = AuthUser {
        email: userinfo.email,
        name: userinfo.name,
        picture: userinfo.picture,
    };

    if let Err(e) = session.insert(SESSION_USER_KEY, &user).await {
        tracing::error!("Failed to store user in session: {}", e);
        return Redirect::to("/admin?error=session_error").into_response();
    }

    tracing::info!("User logged in: {}", user.email);
    Redirect::to("/admin").into_response()
}

/// Logout - clear session
pub async fn auth_logout(session: Session) -> impl IntoResponse {
    let _ = session.remove::<AuthUser>(SESSION_USER_KEY).await;
    Redirect::to("/")
}

/// Get current user from session (internal)
pub async fn get_current_user(session: &Session) -> Option<AuthUser> {
    session.get(SESSION_USER_KEY).await.unwrap_or(None)
}

/// Check if user is authenticated (internal)
#[allow(dead_code)]
pub async fn is_authenticated(session: &Session) -> bool {
    get_current_user(session).await.is_some()
}

/// Check if the given email is in the ADMIN_EMAILS allowlist.
fn is_admin_email(email: &str) -> bool {
    use super::config::SERVER_CONFIG;

    SERVER_CONFIG
        .admin_emails
        .split(',')
        .any(|e| e.trim().eq_ignore_ascii_case(email))
}

/// Middleware that requires authentication for admin paths.
///
/// - `/api/admin/` → 401 Unauthorized if not authenticated, 403 Forbidden if not admin
/// - `/admin` or `/admin/` → redirect to `/auth/google` if not authenticated, 403 if not admin
pub async fn require_admin_auth(
    session: Session,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let path = request.uri().path();
    let is_admin_path = path == "/admin" || path.starts_with("/admin/");
    let is_admin_api = path.starts_with("/api/admin/");

    if is_admin_path || is_admin_api {
        match get_current_user(&session).await {
            None => {
                if is_admin_api {
                    return StatusCode::UNAUTHORIZED.into_response();
                }
                return Redirect::to("/auth/google").into_response();
            }
            Some(user) => {
                if !is_admin_email(&user.email) {
                    tracing::warn!("Non-admin session detected, flushing: {}", user.email);
                    let _ = session.flush().await;
                    return StatusCode::FORBIDDEN.into_response();
                }
            }
        }

        let mut response = next.run(request).await;
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static(
                "no-store, no-cache, must-revalidate, max-age=0, private",
            ),
        );
        response.headers_mut().insert(
            axum::http::header::CDN_CACHE_CONTROL,
            axum::http::HeaderValue::from_static(
                "no-store, no-cache, must-revalidate, max-age=0, private",
            ),
        );
        return response;
    }

    next.run(request).await
}

/// Create auth routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/google", get(auth_google))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/logout", get(auth_logout))
}
