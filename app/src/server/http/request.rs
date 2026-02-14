/// features=local Cookieが設定されているかを判定する
/// NOTE: DB記事表示を一般公開するため、常にtrueを返す
pub(crate) async fn is_local_features() -> bool {
    true
}

/// features=local Cookieが設定されているかを同期的に判定する
/// NOTE: DB記事表示を一般公開するため、常にtrueを返す
pub(crate) fn is_local_features_sync() -> bool {
    true
}

/// SSRリクエストかどうかを判定する
/// Accept: text/html を含む場合はSSR（ブラウザからの直接アクセス）
pub(crate) async fn is_ssr_request() -> bool {
    if let Ok(headers) = leptos_axum::extract::<axum::http::HeaderMap>().await {
        headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("text/html"))
            .unwrap_or(false)
    } else {
        false
    }
}
