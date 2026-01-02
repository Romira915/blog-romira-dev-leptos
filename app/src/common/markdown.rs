use comrak::{Options, markdown_to_html};

/// MarkdownをHTMLに変換
pub fn convert_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.header_ids = None;

    markdown_to_html(markdown, &options)
}
