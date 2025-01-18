use crate::front::components::header::Header;
use crate::front::components::not_found::NotFound;
use leptos::prelude::*;

#[component]
pub(crate) fn NotFoundPage() -> impl IntoView {
    view! {
        <Header is_h1=false />
        <NotFound />
    }
}
