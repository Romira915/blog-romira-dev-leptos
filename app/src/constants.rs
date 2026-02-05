pub(crate) const WEB_APP_TITLE: &str = "Romira's develop blog";
pub(crate) const WEB_APP_DESCRIPTION: &str =
    "Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．";
pub(crate) const ORIGIN: &str = "https://blog.romira.dev";
pub(crate) const WEB_TOP_PAGE_OG_IMAGE_URL: &str =
    "https://blog-romira.imgix.net/46cea3d7-14ce-45bf-9d1e-52d1df39f2d2/romira'sdevelopblog.png";
pub(crate) const NEWT_CDN_BASE_URL: &str = "https://blog-romira-dev.cdn.newt.so/v1";
pub(crate) const NEWT_BASE_URL: &str = "https://blog-romira-dev.api.newt.so/v1";
/// Newt上のauthor content id
pub(crate) const ROMIRA_NEWT_AUTHOR_ID: &str = "63fdfa40a6b1cce86529ebcf";
pub(crate) const PRTIMES_WORD_PRESS_BASE_URL: &str = "https://developers.prtimes.jp";
pub(crate) const PRTIMES_WORD_PRESS_AUTHOR_ID: u64 = 204642668;
pub(crate) const THUMBNAIL_NO_IMAGE_URL: &str =
    "https://blog-romira.imgix.net/df46b2ca-5b5c-4847-ba48-650cbae1ae23/no-image.png";
pub(crate) const HOUR: i32 = 3600;
/// JST (UCT+09:00)
pub(crate) const JST_TZ: i32 = 9;
pub(crate) const DATE_DISPLAY_FORMAT: &str = "%Y年%m月%d日";
pub(crate) const ROMIRA_GITHUB_URL: &str = "https://github.com/Romira915";
pub(crate) const ROMIRA_X_URL: &str = "https://x.com/Romira915";
pub(crate) const QIITA_BASE_URL: &str = "https://qiita.com";

/// レスポンシブ画像用のサイズ (srcset用)
#[allow(dead_code)]
pub const IMAGE_WIDTHS: [u32; 3] = [400, 800, 1200];

/// Newt記事IDからDB記事のslugへのマッピング
/// 旧Newt記事URLを新しいDB記事URLにリダイレクトするために使用
pub(crate) fn get_newt_redirect_slug(newt_id: &str) -> Option<&'static str> {
    match newt_id {
        "643a5bed729275004e0392ce" => Some("i-created-a-blog-site-in-rust"),
        "65a38cfdcc42e4d4373f7ef0" => Some("rust-dioxus-axum-ssr"),
        "67602c172e1a9fe4e94472af" => Some("managing-wsl-development-environment-with-ansible"),
        "678b18b1cfd7179e06d90285" => Some("reimplemented-my-blog-site-using-leptos"),
        _ => None,
    }
}
