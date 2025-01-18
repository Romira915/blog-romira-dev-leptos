use crate::front::pages::article_page::ArticlePage;
use crate::front::pages::not_found_page::NotFoundPage;
use crate::front::pages::top_page::TopPage;
use leptos::config::LeptosOptions;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::{HashedStylesheet, Link, MetaTags, Stylesheet, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, StaticSegment, path};

#[cfg(debug_assertions)]
const ASSETS_ROOT: &str = "";
#[cfg(not(debug_assertions))]
const ASSETS_ROOT: &str = std::env!("ASSETS_ROOT");

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="color-scheme" content="dark light" />
                <script async src="https://www.googletagmanager.com/gtag/js?id=G-4P5K3SBG1K" />
                <script async src=format!("{}/google_analytics.js", ASSETS_ROOT) />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() root=ASSETS_ROOT />
                {if cfg!(debug_assertions) {
                    view! { <Stylesheet href="/pkg/blog-romira-dev.css" /> }.into_any()
                } else {
                    view! { <HashedStylesheet id="leptos" options root=ASSETS_ROOT /> }.into_any()
                }}
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Link
            rel="icon"
            href="https://blog-romira.imgix.net/4874cb12-6e50-4aa3-a1f5-541de4ae184c/icon.JPG?w=32&h=32&auto=format&fit=crop&mask=ellipse&q=75"
        />
        <Router>
            <main>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route path=StaticSegment("") view=TopPage ssr=SsrMode::Async />
                    <Route path=path!("/articles/:id") view=ArticlePage ssr=SsrMode::Async />
                </Routes>
            </main>
        </Router>
    }
}
