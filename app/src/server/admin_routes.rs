use axum::Router;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use tracing::instrument;
use uuid::Uuid;

use super::contexts::AppState;

/// 新規記事作成エンドポイント - UUIDを生成してリダイレクト
#[instrument]
pub async fn new_article_redirect() -> impl IntoResponse {
    let new_id = Uuid::now_v7();
    Redirect::to(&format!("/admin/articles/{}", new_id))
}

/// 管理画面用ルートを作成
#[instrument]
pub fn admin_routes() -> Router<AppState> {
    Router::new().route("/admin/articles/new", get(new_article_redirect))
}
