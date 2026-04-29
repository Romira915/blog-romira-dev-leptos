use comrak::{Options, markdown_to_html};

/// MarkdownをHTMLに変換
pub fn convert_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.header_ids = None;
    options.render.r#unsafe = true;

    markdown_to_html(markdown, &options)
}

/// iframe の src として許可するホスト
#[cfg(feature = "ssr")]
const ALLOWED_IFRAME_HOSTS: &[&str] = &[
    "www.youtube.com",
    "www.youtube-nocookie.com",
    "platform.twitter.com",
    "codepen.io",
];

/// HTMLをサニタイズして公開用に安全な形に整える
///
/// `<script>` / `on*` 属性 / `javascript:` URL を除去し、
/// `<img>` のレスポンシブ属性 (srcset / sizes / loading) と
/// 信頼ドメインからの `<iframe>` 埋め込みを許可する。
#[cfg(feature = "ssr")]
pub fn sanitize_html(html: &str) -> String {
    use std::borrow::Cow;
    use std::collections::HashSet;

    let url_schemes: HashSet<&str> = ["http", "https", "mailto"].into_iter().collect();

    ammonia::Builder::default()
        .add_tags(["iframe"])
        .add_tag_attributes("img", ["srcset", "sizes", "loading"])
        .add_tag_attributes(
            "iframe",
            [
                "src",
                "width",
                "height",
                "title",
                "allow",
                "allowfullscreen",
                "loading",
                "referrerpolicy",
                "sandbox",
                "frameborder",
            ],
        )
        .url_schemes(url_schemes)
        .attribute_filter(|element, attribute, value| {
            if element == "iframe" && attribute == "src" {
                let host = url::Url::parse(value)
                    .ok()
                    .and_then(|u| u.host_str().map(str::to_owned));
                match host {
                    Some(h) if ALLOWED_IFRAME_HOSTS.contains(&h.as_str()) => {
                        Some(Cow::Borrowed(value))
                    }
                    _ => None,
                }
            } else {
                Some(Cow::Borrowed(value))
            }
        })
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htmlタグがそのまま通ること() {
        let input =
            r#"<img src="https://example.com/test.jpg" width="800" height="600" alt="test">"#;
        let result = convert_markdown_to_html(input);
        assert!(
            result.contains(r#"<img src="https://example.com/test.jpg""#),
            "img tag was not preserved: {}",
            result
        );
    }

    #[test]
    fn markdown本文中のimgタグが保持されること() {
        let input = "# タイトル\n\nテキスト\n\n<img src=\"https://blog-romira.imgix.net/dev/photo.jpg?w=800&auto=format&q=75\" width=\"1920\" height=\"1080\" loading=\"lazy\" alt=\"test\">\n\n続きのテキスト";
        let result = convert_markdown_to_html(input);
        eprintln!("comrak output: {}", result);
        assert!(
            result.contains("<img src="),
            "img tag was not preserved in markdown body: {}",
            result
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlでscriptタグが除去されること() {
        let input = r#"<p>hello</p><script>alert(1)</script>"#;
        let result = sanitize_html(input);
        assert!(
            !result.contains("<script"),
            "script tag remains: {}",
            result
        );
        assert!(result.contains("<p>hello</p>"));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlでイベント属性が除去されること() {
        let input = r#"<img src="https://example.com/x.jpg" onerror="alert(1)" alt="x">"#;
        let result = sanitize_html(input);
        assert!(!result.contains("onerror"), "onerror remains: {}", result);
        assert!(result.contains(r#"src="https://example.com/x.jpg""#));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlでjavascriptスキームが除去されること() {
        let input = r#"<a href="javascript:alert(1)">click</a>"#;
        let result = sanitize_html(input);
        assert!(
            !result.contains("javascript:"),
            "javascript scheme remains: {}",
            result
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlでimgのレスポンシブ属性が保持されること() {
        let input = r#"<img src="https://blog-romira.imgix.net/x.jpg?w=800" srcset="https://blog-romira.imgix.net/x.jpg?w=400 400w, https://blog-romira.imgix.net/x.jpg?w=800 800w" sizes="(max-width: 800px) 800px" loading="lazy" alt="x">"#;
        let result = sanitize_html(input);
        assert!(result.contains("srcset="), "srcset stripped: {}", result);
        assert!(result.contains("sizes="), "sizes stripped: {}", result);
        assert!(
            result.contains(r#"loading="lazy""#),
            "loading stripped: {}",
            result
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlで信頼ドメインのiframeが保持されること() {
        let input = r#"<iframe src="https://www.youtube.com/embed/dQw4w9WgXcQ" width="560" height="315" allowfullscreen></iframe>"#;
        let result = sanitize_html(input);
        assert!(result.contains("<iframe"), "iframe stripped: {}", result);
        assert!(
            result.contains(r#"src="https://www.youtube.com/embed/dQw4w9WgXcQ""#),
            "src stripped: {}",
            result
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn sanitize_htmlで未知ドメインのiframeのsrcが除去されること() {
        let input =
            r#"<iframe src="https://evil.example.com/x" width="560" height="315"></iframe>"#;
        let result = sanitize_html(input);
        assert!(
            !result.contains("evil.example.com"),
            "untrusted host remains: {}",
            result
        );
    }
}
