use leptos::prelude::*;
use stylance::import_style;

import_style!(pub header_style, "header.module.scss");

#[component]
pub(crate) fn Header() -> impl IntoView {
    view! {
        <header class=header_style::header>
            <a href="/">
                <h1 class=header_style::blog_title>"Romira's develop blog"</h1>
            </a>
        </header>
    }
}
