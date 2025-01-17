pub(crate) fn set_top_page_cache_control() {
    #[cfg(feature = "ssr")]
    crate::server::http::response::set_top_page_cache_control();
}

pub(crate) fn set_article_page_cache_control() {
    #[cfg(feature = "ssr")]
    crate::server::http::response::set_article_page_cache_control();
}
