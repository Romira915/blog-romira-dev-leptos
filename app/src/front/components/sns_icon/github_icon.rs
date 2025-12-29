use leptos::prelude::*;

use super::sns_icon_style;

#[component]
pub(crate) fn GitHubIcon() -> impl IntoView {
    view! { <img src="https://github.com/github.png" alt="GitHub" class=sns_icon_style::github_icon /> }
}
