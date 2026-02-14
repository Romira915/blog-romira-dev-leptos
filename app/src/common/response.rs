#[cfg(feature = "ssr")]
use leptos::prelude::expect_context;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use std::sync::atomic::{AtomicBool, Ordering};

/// キャッシュコントロールが設定済みかどうかを追跡するフラグ
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct CacheControlSet(pub Arc<AtomicBool>);

#[cfg(feature = "ssr")]
impl CacheControlSet {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn mark_set(&self) {
        self.0.store(true, Ordering::Release);
    }
}

#[cfg(feature = "ssr")]
impl Default for CacheControlSet {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn set_article_page_cache_control(_slug: &str) {
    #[cfg(feature = "ssr")]
    crate::server::http::response::set_article_page_cache_control(
        &expect_context::<ResponseOptions>(),
        _slug,
    );
}

/// トップページ用のキャッシュ設定（既に設定済みならスキップ）
pub(crate) fn set_top_page_cache_control() {
    #[cfg(feature = "ssr")]
    {
        let cache_control_set = expect_context::<CacheControlSet>();
        if cache_control_set.is_set() {
            return;
        }
        cache_control_set.mark_set();
        crate::server::http::response::set_top_page_cache_control(
            &expect_context::<ResponseOptions>(),
        );
    }
}

/// features=local 用のキャッシュ無効化設定（既に設定済みならスキップ）
pub(crate) fn set_feature_page_cache_control() {
    #[cfg(feature = "ssr")]
    {
        let cache_control_set = expect_context::<CacheControlSet>();
        if cache_control_set.is_set() {
            return;
        }
        cache_control_set.mark_set();
        crate::server::http::response::set_preview_article_page_cache_control(&expect_context::<
            ResponseOptions,
        >());
    }
}
