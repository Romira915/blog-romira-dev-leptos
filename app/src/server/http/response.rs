use axum::http::HeaderValue;
use axum::http::header::{CACHE_CONTROL, CDN_CACHE_CONTROL};
use blog_romira_dev_cms::CmsError;
use leptos::prelude::ServerFnError;
use leptos_axum::ResponseOptions;
use reqwest::StatusCode;

/// CmsErrorをStatusCodeにマッピング
pub fn status_code_from_cms_error(error: &CmsError) -> StatusCode {
    match error {
        CmsError::ValidationError(_) => StatusCode::BAD_REQUEST,
        CmsError::NotFound => StatusCode::NOT_FOUND,
        CmsError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// CmsErrorをServerFnErrorに変換し、適切なステータスコードを設定
pub fn cms_error_to_response(response: &ResponseOptions, error: CmsError) -> ServerFnError {
    response.set_status(status_code_from_cms_error(&error));
    ServerFnError::new(error.to_string())
}

pub(crate) fn set_top_page_cache_control(response: &ResponseOptions) {
    response.insert_header(
        CACHE_CONTROL,
        HeaderValue::from_static(
            "no-cache, must-revalidate, max-age=10, stale-while-revalidate=1296000",
        ),
    );
    response.insert_header(
        CDN_CACHE_CONTROL,
        HeaderValue::from_static("max-age=1296000, stale-while-revalidate=1296000"),
    );
}

pub(crate) fn set_article_page_cache_control(response: &ResponseOptions) {
    response.insert_header(
        CACHE_CONTROL,
        HeaderValue::from_static(
            "no-cache, must-revalidate, max-age=10, stale-while-revalidate=1296000",
        ),
    );
    response.insert_header(
        CDN_CACHE_CONTROL,
        HeaderValue::from_static("max-age=1296000, stale-while-revalidate=1296000"),
    );
}

pub(crate) fn set_preview_article_page_cache_control(response: &ResponseOptions) {
    response.insert_header(
        CACHE_CONTROL,
        HeaderValue::from_static(
            "no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private",
        ),
    );
    response.insert_header(
        CDN_CACHE_CONTROL,
        HeaderValue::from_static(
            "no-cache, must-revalidate, no-store, max-age=0, stale-while-revalidate=0, private",
        ),
    );
}
