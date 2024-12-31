use crate::error::HomePageError;
use crate::server::models::newt_article::{NewtArticle, NewtArticleCollection};
use crate::server::services::get_newt_articles;
use crate::SERVER_CONFIG;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

stylance::import_style!(pub my_style, "home_page.module.scss");

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    tracing::info!("HomePage {}", SERVER_CONFIG.new_relic_license_key);

    let articles = Resource::new(
        || (),
        |_| async move {
            get_newt_articles_handler()
                .await
                .expect("failed to get_newt_articles_handler")
        },
    );
    let number = Resource::new(|| (), |_| async move { get_number().await.unwrap() });

    view! {
        <HomePageMeta/>
        <h1>"HomePage"</h1>
        <p>"This is the home page."</p>
        <Suspense fallback=|| "Loading...">
        {move || {
            articles.map(|articles| {
                articles.items.iter().map(|article| {
                    view! {
                        <h2>{article.title.clone()}</h2>
                    }
                }).collect_view()
            })
        }}
        </Suspense>
    }
}

#[component]
pub(crate) fn HomePageMeta() -> impl IntoView {
    view! {
        <Title text="Romira's develop blog"/>
        <Meta property="og:title" content="Romira's develop blog"/>
        <Meta property="og:description" content="Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．"/>
        <Meta property="og:type" content="article"/>
        <Meta property="og:url" content="https://blog.romira.dev"/>
        <Meta property="og:site_name" content="Romira's develop blog"/>
        <Meta property="og:image" content="https://blog-romira.imgix.net/46cea3d7-14ce-45bf-9d1e-52d1df39f2d2/romira'sdevelopblog.png"/>
        <Meta name="twitter:creator" content="@Romira915"/>
    }
}

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}

#[server]
pub(crate) async fn get_newt_articles_handler() -> Result<NewtArticleCollection, ServerFnError> {
    tracing::info!("get_newt_articles");
    let articles = get_newt_articles(reqwest::Client::new(), false).await;
    let articles = match articles {
        Ok(articles) => articles,
        Err(err) => {
            tracing::error!("get_newt_articles: {:?}", err);
            return Err(ServerFnError::from(err));
        }
    };

    Ok(articles)
}
