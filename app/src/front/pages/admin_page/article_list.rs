use leptos::prelude::*;
use leptos_router::components::A;
use stylance::import_style;

use super::AdminLayout;
use crate::common::handlers::admin::{
    DeleteArticleInput, delete_article_handler, get_admin_articles_handler,
};

import_style!(style, "article_list.module.scss");

#[component]
pub fn ArticleListPage() -> impl IntoView {
    // TODO: Fetch articles from server
    let articles = Resource::new(|| (), |_| async move { get_admin_articles_handler().await });

    view! {
        <AdminLayout>
            <div class=style::container>
                <header class=style::header>
                    <h1>"記事一覧"</h1>
                    <a href="/admin/articles/new" rel="external" class=style::new_button>
                        "新規作成"
                    </a>
                </header>
                <Suspense fallback=move || {
                    view! { <p>"読み込み中..."</p> }
                }>
                    {move || {
                        articles
                            .get()
                            .map(|result| {
                                match result {
                                    Ok(list) => {
                                        view! {
                                            <table class=style::table>
                                                <thead>
                                                    <tr>
                                                        <th>"タイトル"</th>
                                                        <th>"ステータス"</th>
                                                        <th>"公開日"</th>
                                                        <th>"操作"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {list
                                                        .into_iter()
                                                        .map(|article| {
                                                            let article_id = article.id.clone();
                                                            let is_draft = article.is_draft;
                                                            view! {
                                                                <tr>
                                                                    <td>{article.title}</td>
                                                                    <td>
                                                                        <span class=if article.is_draft {
                                                                            style::status_draft
                                                                        } else {
                                                                            style::status_published
                                                                        }>
                                                                            {if article.is_draft { "下書き" } else { "公開" }}
                                                                        </span>
                                                                    </td>
                                                                    <td>
                                                                        {article.published_at.unwrap_or_else(|| "-".to_string())}
                                                                    </td>
                                                                    <td>
                                                                        <A
                                                                            href=format!("/admin/articles/{}", article.id)
                                                                            attr:class=style::edit_link
                                                                        >
                                                                            "編集"
                                                                        </A>
                                                                        " "
                                                                        <button
                                                                            class=style::delete_button
                                                                            on:click=move |_| {
                                                                                #[cfg(feature = "hydrate")]
                                                                                {
                                                                                    let confirmed = web_sys::window()
                                                                                        .and_then(|w| {
                                                                                            w.confirm_with_message(
                                                                                                    "この記事を削除しますか？",
                                                                                                )
                                                                                                .ok()
                                                                                        })
                                                                                        .unwrap_or(false);
                                                                                    if confirmed {
                                                                                        let id = article_id.clone();
                                                                                        leptos::task::spawn_local(async move {
                                                                                            let result = delete_article_handler(DeleteArticleInput {
                                                                                                    id,
                                                                                                    is_draft,
                                                                                                })
                                                                                                .await;
                                                                                            match result {
                                                                                                Ok(_) => {
                                                                                                    articles.refetch();
                                                                                                }
                                                                                                Err(e) => {
                                                                                                    #[cfg(feature = "hydrate")]
                                                                                                    if let Some(w) = web_sys::window() {
                                                                                                        let _ = w
                                                                                                            .alert_with_message(&format!("削除エラー: {}", e));
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                }
                                                                            }
                                                                        >
                                                                            "削除"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </tbody>
                                            </table>
                                        }
                                            .into_any()
                                    }
                                    Err(e) => {
                                        view! {
                                            <p class=style::error>{format!("エラー: {}", e)}</p>
                                        }
                                            .into_any()
                                    }
                                }
                            })
                    }}
                </Suspense>
            </div>
        </AdminLayout>
    }
}
