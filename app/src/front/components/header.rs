use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) header_style, "header.module.scss");

#[component]
pub(crate) fn Header(is_h1: bool) -> impl IntoView {
    view! {
        <header class=header_style::header>
            <a href="/" class=header_style::blog_title_link>
                {if is_h1 {
                    view! { <h1 class=header_style::blog_title>"Romira's develop blog"</h1> }
                        .into_any()
                } else {
                    view! { <div class=header_style::blog_title>"Romira's develop blog"</div> }
                        .into_any()
                }}
            </a>
        </header>
    }
}
