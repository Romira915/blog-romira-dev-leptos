use crate::common::handlers::{get_articles_handler, get_author_handler, get_number};
use crate::constants::{ROMIRA_GITHUB_URL, ROMIRA_X_URL};
use crate::front::components::article_card::ArticleCardList;
use crate::front::components::author_card::AuthorCard;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

stylance::import_style!(pub home_page_style, "home_page.module.scss");

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let articles = Resource::new(|| (), |_| async move { get_articles_handler().await });
    let author = Resource::new(|| (), |_| async move { get_author_handler().await });
    let number = Resource::new(|| (), |_| async move { get_number().await.unwrap() });

    view! {
        <HomePageMeta />
        <section class=home_page_style::home_page>
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
pub(crate) fn HomePageMeta() -> impl IntoView {
    view! {
        <Title text="Romira's develop blog" />
        <Meta property="og:title" content="Romira's develop blog" />
        <Meta
            property="og:description"
            content="Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．"
        />
        <Meta property="og:type" content="article" />
        <Meta property="og:url" content="https://blog.romira.dev" />
        <Meta property="og:site_name" content="Romira's develop blog" />
        <Meta
            property="og:image"
            content="https://blog-romira.imgix.net/46cea3d7-14ce-45bf-9d1e-52d1df39f2d2/romira'sdevelopblog.png"
        />
        <Meta name="twitter:creator" content="@Romira915" />
    }
}
