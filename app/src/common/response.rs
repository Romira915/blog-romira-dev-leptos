#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

pub(crate) fn set_top_page_cache_control() {
    #[cfg(feature = "ssr")]
    crate::server::http::response::set_top_page_cache_control(&expect_context::<ResponseOptions>());
}

pub(crate) fn set_article_page_cache_control() {
    #[cfg(feature = "ssr")]
    crate::server::http::response::set_article_page_cache_control(
        &expect_context::<ResponseOptions>(),
    );
}
