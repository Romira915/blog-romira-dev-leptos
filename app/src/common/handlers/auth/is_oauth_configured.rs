use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use tracing::instrument;

/// Server function to check if OAuth is configured
#[instrument]
#[server(input = GetUrl, endpoint = "auth/configured")]
pub async fn is_oauth_configured() -> Result<bool, ServerFnError> {
    Ok(true)
}
