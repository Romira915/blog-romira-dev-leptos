use crate::common::handlers::{get_articles_handler, get_author_handler};
use crate::common::response::{set_feature_page_cache_control, set_top_page_cache_control};
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

    let features = move || query.read().get("features").map(|s| s.to_string());
    let show_local = move || features() == Some("local".to_string());

    // キャッシュコントロールを設定（既に設定済みならスキップ）
    if show_local() {
        set_feature_page_cache_control();
    } else {
        set_top_page_cache_control();
    }

    let articles = Resource::new(features, |features| async move {
        get_articles_handler(features).await
    });
    let author = Resource::new(features, |features| async move {
        get_author_handler(features).await
    });

    view! {
        <TopPageMeta />
        <Show when=show_local>
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
