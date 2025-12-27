use axum::{
    Router,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum::extract::Query;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, RedirectUrl, Scope, StandardRevocableToken, TokenResponse, TokenUrl,
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
};
use serde::Deserialize;
use tower_sessions::Session;

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

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
fn create_oauth_client() -> Option<GoogleOAuthClient> {
    let client_id = SERVER_CONFIG.google_client_id.as_ref()?;
    let client_secret = SERVER_CONFIG.google_client_secret.as_ref()?;
    let app_url = SERVER_CONFIG.app_url.as_ref()?;

    let redirect_url = format!("{}/auth/callback", app_url);

    let client = BasicClient::new(ClientId::new(client_id.clone()))
        .set_client_secret(ClientSecret::new(client_secret.clone()))
        .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string()).unwrap())
        .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    Some(client)
}

/// Start OAuth flow - redirect to Google
pub async fn auth_google(session: Session) -> impl IntoResponse {
    let Some(client) = create_oauth_client() else {
        return Redirect::to("/admin?error=oauth_not_configured").into_response();
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // Store CSRF token in session
    if let Err(e) = session.insert(SESSION_CSRF_KEY, csrf_token.secret().clone()).await {
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

    let Some(client) = create_oauth_client() else {
        return Redirect::to("/admin?error=oauth_not_configured").into_response();
    };

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

/// Create auth routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/google", get(auth_google))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/logout", get(auth_logout))
}

/// Server function to get current authenticated user
#[leptos::prelude::server(endpoint = "auth/me")]
pub async fn get_auth_user() -> Result<Option<AuthUser>, leptos::prelude::ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await?;
    Ok(get_current_user(&session).await)
}

/// Server function to check if OAuth is configured
#[leptos::prelude::server(endpoint = "auth/configured")]
pub async fn is_oauth_configured() -> Result<bool, leptos::prelude::ServerFnError> {
    Ok(SERVER_CONFIG.google_client_id.is_some()
        && SERVER_CONFIG.google_client_secret.is_some()
        && SERVER_CONFIG.app_url.is_some())
}
