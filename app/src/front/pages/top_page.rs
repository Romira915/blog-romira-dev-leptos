use crate::common::handlers::{get_articles_handler, get_author_handler};
use crate::constants::{
    ORIGIN, ROMIRA_GITHUB_URL, ROMIRA_X_URL, WEB_APP_DESCRIPTION, WEB_APP_TITLE,
    WEB_TOP_PAGE_OG_IMAGE_URL,
};
use crate::front::components::article_card::ArticleCardList;
use crate::front::components::author_card::AuthorCard;
use crate::front::components::header::Header;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

stylance::import_style!(pub(crate) top_page_style, "top_page.module.scss");

#[component]
pub(crate) fn TopPage() -> impl IntoView {
    let articles = Resource::new(|| (), |_| async move { get_articles_handler().await });
    let author = Resource::new(|| (), |_| async move { get_author_handler().await });

    view! {
        <TopPageMeta />
        <Header is_h1=true />
        <section class=top_page_style::home_page>
            <Suspense fallback=|| {
                "Loading..."
            }>
                {move || {
                    articles
                        .map(|articles| {
                            match articles {
                                Ok(articles) => {
                                    view! { <ArticleCardList articles=articles.clone() /> }
                                        .into_any()
                                }
                                Err(e) => view! { <p>{format!("Error: {:?}", e)}</p> }.into_any(),
                            }
                        })
                }}
            </Suspense>
            <Suspense fallback=|| {
                "Loading..."
            }>
                {move || {
                    author
                        .map(|author| {
                            match author {
                                Ok(author) => {
                                    view! {
                                        <AuthorCard
                                            author=author.clone()
                                            github_url=ROMIRA_GITHUB_URL
                                            x_url=ROMIRA_X_URL
                                        />
                                    }
                                        .into_any()
                                }
                                Err(e) => view! { <p>{format!("Error: {:?}", e)}</p> }.into_any(),
                            }
                        })
                }}
            </Suspense>
        </section>
    }
}

#[component]
pub(crate) fn TopPageMeta() -> impl IntoView {
    view! {
        <Title text=WEB_APP_TITLE />
        <Meta name="description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:title" content=WEB_APP_TITLE />
        <Meta property="og:description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:type" content="website" />
        <Meta property="og:url" content=ORIGIN />
        <Meta property="og:site_name" content=WEB_APP_TITLE />
        <Meta property="og:image" content=WEB_TOP_PAGE_OG_IMAGE_URL />
        <Meta name="twitter:creator" content="@Romira915" />
    }
}
