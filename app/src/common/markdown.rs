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
}
