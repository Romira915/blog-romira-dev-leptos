use crate::front::components::header::Header;
use crate::front::pages::home_page::HomePage;
use leptos::config::LeptosOptions;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, StaticSegment};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/blog-romira-dev.css"/>
        <Meta name="color-scheme" content="dark light"/>
        <Router>
            <Header/>
            <main>
                <Routes fallback=|| "Not Found.">
                    <Route path=StaticSegment("") view=HomePage ssr=SsrMode::Async/>
                </Routes>
            </main>
        </Router>
    }
}
