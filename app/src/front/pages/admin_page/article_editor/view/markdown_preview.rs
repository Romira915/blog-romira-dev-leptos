use leptos::prelude::*;

use super::style;
use crate::common::markdown::convert_markdown_to_html;
use crate::front::components::article_detail::article_body_style;

#[component]
pub fn MarkdownPreview(content: RwSignal<String>) -> impl IntoView {
    let html_content = move || convert_markdown_to_html(&content.get());

    view! {
        <div
            class=format!("{} {}", article_body_style::markdown_body, style::preview_content)
            inner_html=html_content
        ></div>
    }
}
