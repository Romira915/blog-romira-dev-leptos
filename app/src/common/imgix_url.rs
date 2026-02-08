/// imgix URLかどうか判定する
pub fn is_imgix_url(url: &str) -> bool {
    url.contains(".imgix.net/")
}

/// URLからクエリパラメータを除去してベースURLを取得する
pub fn extract_base_url(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// ベースURLと幅リストからsrcset文字列を生成する
pub fn generate_srcset(base_url: &str, widths: &[u32]) -> String {
    widths
        .iter()
        .map(|w| format!("{base_url}?w={w}&auto=format&q=75 {w}w"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imgix_urlを判定できる() {
        assert!(is_imgix_url(
            "https://blog-romira.imgix.net/dev/path/photo.jpg"
        ));
        assert!(is_imgix_url(
            "https://blog-romira.imgix.net/dev/path/photo.jpg?w=800&auto=format"
        ));
        assert!(!is_imgix_url("https://example.com/image.jpg"));
        assert!(!is_imgix_url(
            "https://blog-romira-dev.cdn.newt.so/v1/image.jpg"
        ));
    }

    #[test]
    fn ベースurlを抽出できる() {
        assert_eq!(
            extract_base_url("https://blog-romira.imgix.net/dev/photo.jpg?w=800&auto=format"),
            "https://blog-romira.imgix.net/dev/photo.jpg"
        );
        assert_eq!(
            extract_base_url("https://blog-romira.imgix.net/dev/photo.jpg"),
            "https://blog-romira.imgix.net/dev/photo.jpg"
        );
    }

    #[test]
    fn srcset文字列を生成できる() {
        let base = "https://blog-romira.imgix.net/dev/photo.jpg";
        let srcset = generate_srcset(base, &[400, 800, 1200]);
        assert_eq!(
            srcset,
            "https://blog-romira.imgix.net/dev/photo.jpg?w=400&auto=format&q=75 400w, \
             https://blog-romira.imgix.net/dev/photo.jpg?w=800&auto=format&q=75 800w, \
             https://blog-romira.imgix.net/dev/photo.jpg?w=1200&auto=format&q=75 1200w"
        );
    }

    #[test]
    fn 空の幅リストで空文字列を返す() {
        let base = "https://blog-romira.imgix.net/dev/photo.jpg";
        assert_eq!(generate_srcset(base, &[]), "");
    }
}
