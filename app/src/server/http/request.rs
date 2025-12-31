/// リクエストのCookieから features の値を取得する
pub(crate) async fn get_features_cookie() -> Option<String> {
    let headers = leptos_axum::extract::<axum::http::HeaderMap>().await.ok()?;
    let cookie_header = headers.get(axum::http::header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    // "features=value" 形式のCookieを探す
    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("features=") {
            return Some(value.to_string());
        }
    }
    None
}

/// features=local Cookieが設定されているかを判定する
pub(crate) async fn is_local_features() -> bool {
    get_features_cookie().await.as_deref() == Some("local")
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
