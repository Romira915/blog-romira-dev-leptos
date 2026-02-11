use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use tracing::instrument;

#[instrument]
#[server(input = GetUrl, endpoint = "admin/get_categories")]
pub async fn get_categories_handler() -> Result<Vec<String>, ServerFnError> {
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let categories = state
        .category_service()
        .fetch_all()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(categories.into_iter().map(|c| c.name).collect())
}
