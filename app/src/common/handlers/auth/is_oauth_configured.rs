use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use tracing::instrument;

/// Server function to check if OAuth is configured
#[instrument]
#[server(input = GetUrl, endpoint = "auth/configured")]
pub async fn is_oauth_configured() -> Result<bool, ServerFnError> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
    Ok(!client_id.trim().is_empty() && !client_secret.trim().is_empty())
}
