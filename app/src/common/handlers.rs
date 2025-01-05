use leptos::prelude::*;
use leptos::prelude::{expect_context, ServerFnError};

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}

#[server]
pub(crate) async fn get_newt_articles_handler()
    -> Result<(), ServerFnError> {
    use crate::constants::{NEWT_BASE_URL, NEWT_CDN_BASE_URL};
    use crate::server::services::NewtArticleService;

    let app_state = expect_context::<crate::AppState>();
    tracing::info!("get_newt_articles {}", app_state.leptos_options.output_name);
    let service = NewtArticleService::new(reqwest::Client::new(), NEWT_CDN_BASE_URL, NEWT_BASE_URL);
    let articles = service.get_newt_articles(false).await;
    let articles = match articles {
        Ok(articles) => articles,
        Err(err) => {
            tracing::error!("get_newt_articles: {:?}", err);
            return Err(ServerFnError::from(err));
        }
    };

    // Ok(articles)
    Ok(())
}
