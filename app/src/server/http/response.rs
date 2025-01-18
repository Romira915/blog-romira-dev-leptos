use axum::http::HeaderValue;
use axum::http::header::{CACHE_CONTROL, CDN_CACHE_CONTROL};
use leptos_axum::ResponseOptions;

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
