use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) not_found_style, "not_found.module.scss");

#[component]
pub(crate) fn NotFound() -> impl IntoView {
    view! {
        <section aria-labelledby="not-found" class=not_found_style::not_found_container>
            <h1 not_found_style::not_found>Not Found</h1>
            <p class=not_found_style::not_found_description>
                {"ページが見つかりませんでした。"}
            </p>
        </section>
    }
}
