//noinspection NonAsciiCharacters
//! Admin handlers integration tests
//!
//! Leptosのserver functionをHTTPリクエストレベルでテスト
//!
//! GETおよびPOST（JSON形式）のハンドラをテスト

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use blog_romira_dev_app::{App, AppState};
use http_body_util::BodyExt;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use serde_json::json;
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
    AppState::new_for_test(conf.leptos_options, pool)
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

    let nonexistent_id = Uuid::now_v7();
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

// =====================================
// save_draft_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_save_draft_新規作成の場合記事idを返すこと(pool: PgPool) {
    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    // Upsert方式では事前にクライアント側でUUIDを生成する
    let new_id = Uuid::now_v7();

    // Leptosサーバー関数はパラメータ名をJSONフィールド名として使用する
    let input = json!({
        "input": {
            "id": new_id.to_string(),
            "title": "New Draft",
            "slug": "new-draft",
            "body": "New Body",
            "description": null
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/save_draft")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let article_id: String = serde_json::from_slice(&body).unwrap();

    // 返されたIDが渡したIDと一致することを確認
    assert_eq!(article_id, new_id.to_string());

    // 返されたIDで記事が取得できることを確認
    let uuid = Uuid::parse_str(&article_id).expect("Valid UUID should be returned");
    let article = sqlx::query!("SELECT title FROM draft_articles WHERE id = $1", uuid)
        .fetch_one(&pool)
        .await
        .expect("Article should exist");
    assert_eq!(article.title, "New Draft");
}

#[sqlx::test(migrations = "../migrations")]
async fn test_save_draft_更新の場合記事が更新されること(pool: PgPool) {
    let draft_id =
        insert_draft_article(&pool, "Original Title", "original-slug", "Original Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": draft_id.to_string(),
            "title": "Updated Title",
            "slug": "updated-slug",
            "body": "Updated Body",
            "description": "Updated Description"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/save_draft")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 記事が更新されていることを確認
    let article = sqlx::query!(
        "SELECT title, slug, body FROM draft_articles WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .expect("Article should exist");
    assert_eq!(article.title, "Updated Title");
    assert_eq!(article.slug, "updated-slug");
    assert_eq!(article.body, "Updated Body");
}

// =====================================
// save_published_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_save_published_正常系_記事が更新されること(pool: PgPool) {
    let published_id =
        insert_published_article(&pool, "Original Title", "original-slug", "Original Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": published_id.to_string(),
            "title": "Updated Title",
            "slug": "updated-slug",
            "body": "Updated Body",
            "description": "Updated Description"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/save_published")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 記事が更新されていることを確認
    let article = sqlx::query!(
        "SELECT title, slug, body FROM published_articles WHERE id = $1",
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Article should exist");
    assert_eq!(article.title, "Updated Title");
    assert_eq!(article.slug, "updated-slug");
}

#[sqlx::test(migrations = "../migrations")]
async fn test_save_published_空タイトルの場合バリデーションエラーを返すこと(
    pool: PgPool,
) {
    let published_id =
        insert_published_article(&pool, "Original Title", "original-slug", "Original Body").await;

    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": published_id.to_string(),
            "title": "",
            "slug": "valid-slug",
            "body": "Body",
            "description": null
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/save_published")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_save_published_空スラッグの場合バリデーションエラーを返すこと(
    pool: PgPool,
) {
    let published_id =
        insert_published_article(&pool, "Original Title", "original-slug", "Original Body").await;

    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": published_id.to_string(),
            "title": "Valid Title",
            "slug": "",
            "body": "Body",
            "description": null
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/save_published")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// =====================================
// publish_article_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_publish_article_正常系_下書きが公開記事になること(pool: PgPool) {
    let draft_id = insert_draft_article(&pool, "Draft Title", "draft-slug", "Draft Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": draft_id.to_string()
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/publish_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let published_id_str: String = serde_json::from_slice(&body).unwrap();
    let published_id = Uuid::parse_str(&published_id_str).expect("Valid UUID should be returned");

    // 公開記事が作成されていることを確認
    let published = sqlx::query!(
        "SELECT title, slug FROM published_articles WHERE id = $1",
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Published article should exist");
    assert_eq!(published.title, "Draft Title");
    assert_eq!(published.slug, "draft-slug");

    // 下書きが削除されていることを確認
    let draft_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM draft_articles WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(draft_count, Some(0));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_publish_article_存在しない下書きの場合notfoundエラーを返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let nonexistent_id = Uuid::now_v7();
    let input = json!({
        "input": {
            "id": nonexistent_id.to_string()
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/publish_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_publish_article_空スラッグの下書きの場合バリデーションエラーを返すこと(
    pool: PgPool,
) {
    let draft_id = insert_draft_article(&pool, "Draft Title", "", "Draft Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": draft_id.to_string()
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/publish_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 下書きは削除されていないことを確認
    let draft_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM draft_articles WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(draft_count, Some(1));
}
