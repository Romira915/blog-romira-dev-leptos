use crate::common::handlers::{get_articles_handler, get_author_handler};
use crate::constants::{ROMIRA_GITHUB_URL, ROMIRA_X_URL};
use crate::front::components::article_card::ArticleCardList;
use crate::front::components::author_card::AuthorCard;
use crate::front::components::header::Header;
use leptos::prelude::*;
use leptos_meta::Meta;
use leptos_router::hooks::use_query_map;

use super::TopPageMeta;
use super::top_page_style;

#[component]
pub(crate) fn TopPage() -> impl IntoView {
    let query = use_query_map();
    let preview = move || query.read().get("preview").map(|s| s.to_string());
    let is_preview = move || preview() == Some("local".to_string());

    let articles = Resource::new(preview, |preview| async move {
        get_articles_handler(preview).await
    });
    let author = Resource::new(|| (), |_| async move { get_author_handler().await });

    view! {
        <TopPageMeta />
        <Show when=is_preview>
            <Meta name="robots" content="noindex, nofollow" />
        </Show>
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
                                Err(e) => view! { <p>{format!("Error: {e:?}")}</p> }.into_any(),
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
                                Err(e) => view! { <p>{format!("Error: {e:?}")}</p> }.into_any(),
                            }
                        })
                }}
            </Suspense>
        </section>
    }
}
