use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use tracing::instrument;

/// Server function to check if OAuth is configured
#[instrument]
#[server(input = GetUrl, endpoint = "auth/configured")]
pub async fn is_oauth_configured() -> Result<bool, ServerFnError> {
    use crate::server::config::SERVER_CONFIG;

    Ok(SERVER_CONFIG.google_client_id.is_some()
        && SERVER_CONFIG.google_client_secret.is_some()
        && SERVER_CONFIG.app_url.is_some())
}
