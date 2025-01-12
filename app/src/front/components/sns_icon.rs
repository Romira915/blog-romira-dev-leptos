use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) sns_icon_style, "sns_icon.module.scss");

#[component]
pub(crate) fn GitHubIcon() -> impl IntoView {
    view! { <img src="https://github.com/github.png" alt="GitHub" class=sns_icon_style::github_icon /> }
}

#[component]
pub(crate) fn XIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 32 32" aria-hidden="true" class=sns_icon_style::x_icon>
            <circle cx="16" cy="16" r="15" class=sns_icon_style::x_icon_circle />
            <g class=sns_icon_style::x_icon_g>
                <path
                    class=sns_icon_style::x_icon_path
                    d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
                ></path>
            </g>
        </svg>
    }
}
