//noinspection NonAsciiCharacters
//! Admin handlers integration tests
//!
//! Leptosのserver functionをHTTPリクエストレベルでテスト
//!
//! 注意: POSTリクエストのテストはLeptosのシリアライゼーション形式の複雑さにより、
//! Service層（cms/tests/services_test.rs）でカバーしています。

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use blog_romira_dev_app::{App, AppState};
use http_body_util::BodyExt;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

/// テスト用のルーターを構築
fn build_test_router(app_state: AppState) -> Router {
    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            {
                let leptos_options = app_state.leptos_options().clone();
                move || blog_romira_dev_app::shell(leptos_options.clone())
            },
        )
        .with_state(app_state)
}

/// テスト用のAppStateを作成
fn create_test_app_state(pool: PgPool) -> AppState {
    let conf = get_configuration(Some("../Cargo.toml")).expect("Failed to get configuration");
    AppState::new(conf.leptos_options, pool)
}

// =====================================
// ヘルパー関数
// =====================================

async fn insert_draft_article(pool: &PgPool, title: &str, slug: &str, body: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO draft_articles (title, slug, body) VALUES ($1, $2, $3) RETURNING id"#,
        title,
        slug,
        body
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert draft article")
}

async fn insert_published_article(pool: &PgPool, title: &str, slug: &str, body: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO published_articles (title, slug, body) VALUES ($1, $2, $3) RETURNING id"#,
        title,
        slug,
        body
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert published article")
}

// =====================================
// get_admin_articles_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_get_admin_articles_記事がない場合空リストを返すこと(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let articles: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(articles.is_empty());
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_admin_articles_下書きと公開記事が混在する場合両方返すこと(
    pool: PgPool,
) {
    // 下書き記事を作成
    insert_draft_article(&pool, "Draft Title", "draft-slug", "Draft Body").await;
    // 公開記事を作成
    insert_published_article(&pool, "Published Title", "published-slug", "Published Body").await;

    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let articles: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(articles.len(), 2);

    // 下書きと公開記事の両方が含まれていることを確認
    let has_draft = articles.iter().any(|a| a["is_draft"] == true);
    let has_published = articles.iter().any(|a| a["is_draft"] == false);
    assert!(has_draft, "下書き記事が含まれていること");
    assert!(has_published, "公開記事が含まれていること");
}

// =====================================
// get_article_for_edit_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_get_article_for_edit_下書き記事が見つかる場合is_draftがtrueであること(
    pool: PgPool,
) {
    let draft_id = insert_draft_article(&pool, "Draft Title", "draft-slug", "Draft Body").await;

    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/admin/get_article?id={}", draft_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let article: Option<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let article = article.expect("Article should be found");

    assert_eq!(article["title"], "Draft Title");
    assert_eq!(article["slug"], "draft-slug");
    assert_eq!(article["is_draft"], true);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_article_for_edit_公開記事が見つかる場合is_draftがfalseであること(
    pool: PgPool,
) {
    let published_id =
        insert_published_article(&pool, "Published Title", "published-slug", "Published Body")
            .await;

    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/admin/get_article?id={}", published_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let article: Option<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let article = article.expect("Article should be found");

    assert_eq!(article["title"], "Published Title");
    assert_eq!(article["is_draft"], false);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_article_for_edit_記事が見つからない場合noneを返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let nonexistent_id = Uuid::new_v4();
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/admin/get_article?id={}", nonexistent_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let article: Option<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(article.is_none());
}

#[sqlx::test(migrations = "../migrations")]
async fn test_get_article_for_edit_無効なuuidの場合エラーを返すこと(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_article?id=invalid-uuid")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // ServerFnErrorはデフォルトで500を返す
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
