use crate::front::pages::admin_page::{ArticleEditorPage, ArticleListPage};
use crate::front::pages::article_page::ArticlePage;
use crate::front::pages::not_found_page::NotFoundPage;
use crate::front::pages::preview_article_page::PreviewArticlePage;
use crate::front::pages::top_page::TopPage;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::{Link, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, StaticSegment, path};

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
                    <Route path=path!("/preview/:id") view=PreviewArticlePage ssr=SsrMode::Async />
                    // Admin routes
                    <Route path=path!("/admin") view=ArticleListPage ssr=SsrMode::Async />
                    <Route path=path!("/admin/articles") view=ArticleListPage ssr=SsrMode::Async />
                    <Route
                        path=path!("/admin/articles/new")
                        view=ArticleEditorPage
                        ssr=SsrMode::Async
                    />
                    <Route
                        path=path!("/admin/articles/:id")
                        view=ArticleEditorPage
                        ssr=SsrMode::Async
                    />
                </Routes>
            </main>
        </Router>
    }
}
