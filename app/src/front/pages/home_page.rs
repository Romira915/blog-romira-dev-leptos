use crate::common::handlers::{get_articles_handler, get_number};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

stylance::import_style!(pub my_style, "home_page.module.scss");

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let articles = Resource::new(|| (), |_| async move { get_articles_handler().await });
    let number = Resource::new(|| (), |_| async move { get_number().await.unwrap() });

    view! {
        <HomePageMeta/>
        <h1>"HomePage 変更テスト3"</h1>
        <p>"This is the home page."</p>
        <Suspense fallback=|| "Loading...">
        {move || {
            articles.map(|articles| {
                match articles {
                    Ok(articles) => {
                        articles.iter().map(|article| {
                            view! {
                                <a href={article.src.clone()} >
                                    <h2>{article.title.clone()}</h2>
                                    <p>{article.published_at.format("%Y年%m月%d日 %H:%M").to_string()}</p>
                                    <p>{article.category.clone()}</p>
                                    <img src={article.thumbnail_url.clone()} alt={article.title.clone()} width=300 height=300 />
                                </a>
                            }
                        }).collect_view().into_any()
                    }
                    Err(e) => {
                        view! {
                            <p>{format!("Error: {:?}", e)}</p>
                        }.into_any()
                    }
                }
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
