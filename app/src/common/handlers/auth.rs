use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Authenticated user information (shared between client and server)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Server function to get current authenticated user
#[instrument]
#[server(input = GetUrl, endpoint = "auth/me")]
pub async fn get_auth_user() -> Result<Option<AuthUser>, ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;

    const SESSION_USER_KEY: &str = "user";

    let session: Session = extract().await?;
    Ok(session.get(SESSION_USER_KEY).await.unwrap_or(None))
}

/// Server function to check if OAuth is configured
#[instrument]
#[server(input = GetUrl, endpoint = "auth/configured")]
pub async fn is_oauth_configured() -> Result<bool, ServerFnError> {
    use crate::server::config::SERVER_CONFIG;

    Ok(SERVER_CONFIG.google_client_id.is_some()
        && SERVER_CONFIG.google_client_secret.is_some()
        && SERVER_CONFIG.app_url.is_some())
}
