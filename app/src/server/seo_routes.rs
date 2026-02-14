use axum::Router;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use blog_romira_dev_cms::PublishedArticleService;
use chrono::Utc;

use crate::constants::ORIGIN;

use super::contexts::AppState;

/// robots.txt を返すハンドラ
async fn robots_txt() -> impl IntoResponse {
    let body = format!(
        "User-agent: *\n\
         Allow: /\n\
         Disallow: /admin/\n\
         Disallow: /preview/\n\
         Disallow: /api/\n\
         \n\
         Sitemap: {}/sitemap.xml\n",
        ORIGIN
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

/// sitemap.xml を動的生成するハンドラ
async fn sitemap_xml(
    State(published_article_service): State<PublishedArticleService>,
) -> impl IntoResponse {
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    // トップページ
    xml.push_str(&format!(
        "  <url>\n\
         \x20   <loc>{}</loc>\n\
         \x20   <lastmod>{}</lastmod>\n\
         \x20   <changefreq>daily</changefreq>\n\
         \x20   <priority>1.0</priority>\n\
         \x20 </url>\n",
        ORIGIN, today
    ));

    // 公開記事一覧
    if let Ok(articles) = published_article_service.fetch_all().await {
        for article_with_cats in &articles {
            let article = &article_with_cats.article;
            let lastmod = article.updated_at.format("%Y-%m-%d");
            xml.push_str(&format!(
                "  <url>\n\
                 \x20   <loc>{}/articles/{}</loc>\n\
                 \x20   <lastmod>{}</lastmod>\n\
                 \x20   <changefreq>monthly</changefreq>\n\
                 \x20   <priority>0.8</priority>\n\
                 \x20 </url>\n",
                ORIGIN, article.slug, lastmod
            ));
        }
    }

    xml.push_str("</urlset>\n");

    (
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (
                header::CACHE_CONTROL,
                "no-cache, must-revalidate, max-age=10, stale-while-revalidate=1296000",
            ),
            (
                header::CDN_CACHE_CONTROL,
                "max-age=1296000, stale-while-revalidate=1296000",
            ),
            (header::HeaderName::from_static("cache-tag"), "sitemap"),
        ],
        xml,
    )
}

/// SEO関連ルートを作成
pub fn seo_routes() -> Router<AppState> {
    Router::new()
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_xml))
}
