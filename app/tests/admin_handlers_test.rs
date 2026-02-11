//noinspection NonAsciiCharacters
//! Admin handlers integration tests
//!
//! Leptosのserver functionをHTTPリクエストレベルでテスト
//!
//! GETおよびPOST（JSON形式）のハンドラをテスト

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use blog_romira_dev_app::common::handlers::auth::AuthUser;
use blog_romira_dev_app::common::response::CacheControlSet;
use blog_romira_dev_app::{App, AppState, auth_routes, require_admin_auth};
use http_body_util::BodyExt;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use tower_sessions::cookie::SameSite;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use uuid::Uuid;

/// テスト用にADMIN_EMAILSを設定（1回だけ実行）
fn ensure_admin_emails_env() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // SAFETY: Called once before any other threads access SERVER_CONFIG
        unsafe {
            std::env::set_var("ADMIN_EMAILS", "test@example.com,admin@example.com");
        }
    });
}

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

/// 認証ミドルウェア付きテスト用ルーターを構築
fn build_test_router_with_auth(app_state: AppState) -> Router {
    ensure_admin_emails_env();

    let routes = generate_route_list(App);

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

    Router::new()
        .route("/test/login", post(test_login_handler))
        .route("/test/login_as", post(test_login_as_handler))
        .merge(auth_routes())
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || {
                    provide_context(app_state.clone());
                    provide_context(CacheControlSet::new());
                }
            },
            {
                let leptos_options = app_state.leptos_options().clone();
                move || blog_romira_dev_app::shell(leptos_options.clone())
            },
        )
        .with_state(app_state)
        .layer(axum::middleware::from_fn(require_admin_auth))
        .layer(session_layer)
}

/// テスト用ログインハンドラ — 許可メール(test@example.com)でセッションに書き込む
async fn test_login_handler(session: Session) -> StatusCode {
    let user = AuthUser {
        email: "test@example.com".to_string(),
        name: Some("Test User".to_string()),
        picture: None,
    };
    session.insert("user", &user).await.unwrap();
    StatusCode::OK
}

/// テスト用ログインハンドラ — 任意メールでセッションに書き込む（JSON body: {"email": "..."}）
async fn test_login_as_handler(
    session: Session,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> StatusCode {
    let email = payload["email"].as_str().unwrap_or("unknown@example.com");
    let user = AuthUser {
        email: email.to_string(),
        name: Some("Test User".to_string()),
        picture: None,
    };
    session.insert("user", &user).await.unwrap();
    StatusCode::OK
}

/// テスト用ヘルパー: 指定メールでログインしてセッションcookieを取得
async fn login_as_and_get_cookie(app: &Router, email: &str) -> String {
    let payload = json!({ "email": email });
    let request = Request::builder()
        .method("POST")
        .uri("/test/login_as")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response
        .headers()
        .get("set-cookie")
        .expect("Login response should have set-cookie header")
        .to_str()
        .unwrap()
        .to_string()
}

/// テスト用ヘルパー: ログインしてセッションcookieを取得
async fn login_and_get_cookie(app: &Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/test/login")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response
        .headers()
        .get("set-cookie")
        .expect("Login response should have set-cookie header")
        .to_str()
        .unwrap()
        .to_string()
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

// =====================================
// delete_article_handler のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_delete_article_下書き記事が正常に削除されること(pool: PgPool) {
    let draft_id = insert_draft_article(&pool, "Delete Draft", "delete-draft", "Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": draft_id.to_string(),
            "is_draft": true
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/delete_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 下書きが削除されていることを確認
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM draft_articles WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, Some(0));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_delete_article_公開記事が正常に削除されること(pool: PgPool) {
    let published_id =
        insert_published_article(&pool, "Delete Published", "delete-published", "Body").await;

    let app_state = create_test_app_state(pool.clone());
    let app = build_test_router(app_state);

    let input = json!({
        "input": {
            "id": published_id.to_string(),
            "is_draft": false
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/delete_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 公開記事が削除されていることを確認
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM published_articles WHERE id = $1",
        published_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, Some(0));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_delete_article_存在しない記事の場合notfoundエラーを返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router(app_state);

    let nonexistent_id = Uuid::now_v7();
    let input = json!({
        "input": {
            "id": nonexistent_id.to_string(),
            "is_draft": true
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/delete_article")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =====================================
// 認証ミドルウェアのテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_未認証でget管理apiにアクセスすると401を返すこと(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_認証済みでget管理apiにアクセスするとリクエストが通ること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // ログインしてセッションcookieを取得
    let cookie = login_and_get_cookie(&app).await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    // 認証が通りハンドラが実行される（200 OK）
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_未認証でpost管理apiにアクセスすると401を返すこと(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let input = json!({
        "input": {
            "id": Uuid::now_v7().to_string(),
            "title": "Test",
            "slug": "test",
            "body": "Test Body",
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_公開apiは認証不要でアクセスできること(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/get_articles_handler")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // 認証なしでもミドルウェアを通過し、ハンドラが実行される（401ではない）
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_未認証で管理ページにアクセスするとログイン画面にリダイレクトされること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/admin")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/google"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn test_未認証で管理サブページにアクセスするとログイン画面にリダイレクトされること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/admin/articles")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/google"
    );
}

// =====================================
// 管理者メールアドレス制限のテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_非許可メールで管理apiにアクセスすると403を返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 非許可メールでログイン
    let cookie = login_as_and_get_cookie(&app, "unauthorized@example.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_非許可メールで管理ページにアクセスすると403を返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 非許可メールでログイン
    let cookie = login_as_and_get_cookie(&app, "unauthorized@example.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/admin")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_非許可メールで管理apiにアクセスするとセッションが削除されること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 非許可メールでログイン
    let cookie = login_as_and_get_cookie(&app, "unauthorized@example.com").await;

    // 1回目: 403 Forbidden（セッションが削除される）
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 2回目: セッションが削除されているので401 Unauthorized
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_許可メールで管理apiにアクセスすると正常に通ること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 許可メール(test@example.com)でログイン
    let cookie = login_as_and_get_cookie(&app, "test@example.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// =====================================
// is_admin_email のテスト（ミドルウェア経由）
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_大文字小文字が異なるメールで管理apiにアクセスすると正常に通ること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 大文字小文字混在メールでログイン（ADMIN_EMAILS=test@example.com に対して）
    let cookie = login_as_and_get_cookie(&app, "TEST@EXAMPLE.COM").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_複数許可メールの2番目で管理apiにアクセスすると正常に通ること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // 2番目の許可メール(admin@example.com)でログイン
    let cookie = login_as_and_get_cookie(&app, "admin@example.com").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// =====================================
// ログアウトのテスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_ログアウトするとルートにリダイレクトされること(pool: PgPool) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let cookie = login_and_get_cookie(&app).await;

    let request = Request::builder()
        .method("GET")
        .uri("/auth/logout")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/"
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn test_ログアウト後に管理apiにアクセスすると401を返すこと(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    let cookie = login_and_get_cookie(&app).await;

    // ログアウト
    let request = Request::builder()
        .method("GET")
        .uri("/auth/logout")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let _ = app.clone().oneshot(request).await.unwrap();

    // ログアウト後に管理APIにアクセス → セッションが消えているので401
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/get_articles")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// =====================================
// パス判定の境界テスト
// =====================================

#[sqlx::test(migrations = "../migrations")]
async fn test_admin接頭辞だが管理パスでないパスは認証不要であること(
    pool: PgPool,
) {
    let app_state = create_test_app_state(pool);
    let app = build_test_router_with_auth(app_state);

    // /adminPanel は /admin でも /admin/ で始まるパスでもないので認証不要
    let request = Request::builder()
        .method("GET")
        .uri("/adminPanel")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // ミドルウェアで401/403にならないこと（ルートがないので他のステータスになる）
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}
