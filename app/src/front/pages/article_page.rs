use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub(crate) fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();

    view! {
        <div>
            <h1>{"ArticlePage"}</h1>
            <p>{format!("id: {}", id)}</p>
        </div>
    }
}
