use leptos::prelude::*;

use super::style;
use crate::front::components::article_detail::article_body_style;

#[component]
pub fn MarkdownPreview(content: RwSignal<String>) -> impl IntoView {
    let html_content = move || {
        use comrak::{Options, markdown_to_html};

        let markdown = content.get();
        let mut options = Options::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.header_ids = None;

        markdown_to_html(&markdown, &options)
    };

    view! {
        <div
            class=format!("{} {}", article_body_style::markdown_body, style::preview_content)
            inner_html=html_content
        ></div>
    }
}
