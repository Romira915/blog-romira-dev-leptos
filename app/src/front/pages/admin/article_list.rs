use leptos::prelude::*;
use leptos_router::components::A;
use stylance::import_style;

use super::AdminLayout;

import_style!(style, "article_list.module.scss");

#[component]
pub fn ArticleListPage() -> impl IntoView {
    // TODO: Fetch articles from server
    let articles = Resource::new(|| (), |_| async move { fetch_admin_articles().await });

    view! {
        <AdminLayout>
            <div class=style::container>
                <header class=style::header>
                    <h1>"記事一覧"</h1>
                    <A href="/admin/articles/new" attr:class=style::new_button>
                        "新規作成"
                    </A>
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
                                                                        {article
                                                                            .published_at
                                                                            .unwrap_or_else(|| "-".to_string())}
                                                                    </td>
                                                                    <td>
                                                                        <A
                                                                            href=format!("/admin/articles/{}", article.id)
                                                                            attr:class=style::edit_link
                                                                        >
                                                                            "編集"
                                                                        </A>
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
                                        view! { <p class=style::error>{format!("エラー: {}", e)}</p> }
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdminArticleListItem {
    pub id: String,
    pub title: String,
    pub is_draft: bool,
    pub published_at: Option<String>,
}

#[server(endpoint = "admin/get_articles")]
pub async fn fetch_admin_articles() -> Result<Vec<AdminArticleListItem>, ServerFnError> {
    use blog_romira_dev_cms::AdminArticleService;
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let articles = AdminArticleService::fetch_all(state.db_pool())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(articles
        .into_iter()
        .map(|a| AdminArticleListItem {
            id: a.id().to_string(),
            title: a.title().to_string(),
            is_draft: a.is_draft(),
            published_at: a.published_at().map(|d| {
                d.format("%Y年%m月%d日").to_string()
            }),
        })
        .collect())
}
