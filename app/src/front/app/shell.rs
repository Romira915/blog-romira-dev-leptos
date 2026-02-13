use leptos::config::LeptosOptions;
use leptos::prelude::*;
use leptos::{IntoView, view};
use leptos_meta::{HashedStylesheet, MetaTags, Stylesheet};

use super::ASSETS_ROOT;
use super::App;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="color-scheme" content="dark light" />
                <meta name="theme-color" content="#1a1a2e" />
                <link rel="dns-prefetch" href="https://blog-romira.imgix.net" />
                <link
                    rel="preconnect"
                    href="https://blog-romira.imgix.net"
                    crossorigin="anonymous"
                />
                <link rel="dns-prefetch" href="https://cdnjs.cloudflare.com" />
                <link
                    rel="preconnect"
                    href="https://cdnjs.cloudflare.com"
                    crossorigin="anonymous"
                />
                <script async src="https://www.googletagmanager.com/gtag/js?id=G-4P5K3SBG1K" />
                <script>
                    {"window.dataLayer = window.dataLayer || [];
                    function gtag(){dataLayer.push(arguments);}
                    gtag('js', new Date());
                    
                    gtag('config', 'G-4P5K3SBG1K');"}
                </script>
                <link rel="stylesheet" href=format!("{}/google.min.css", ASSETS_ROOT) />
                <script
                    src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"
                    integrity="sha512-EBLzUL8XLl+va/zAsmXwS7Z2B1F9HUHkZwyS/VKwh3S7T/U0nF4BaU29EP/ZSf6zgiIxYAnKLu6bJ8dqpmX5uw=="
                    crossorigin="anonymous"
                    referrerpolicy="no-referrer"
                ></script>
                <script
                    src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/powershell.min.js"
                    integrity="sha512-J5RVPDNPGNQPjzhjtYPsrLHTX/7GPhPU1AvmyCo/hjKWBecwfX6ByD0oG/NAS5VU3hwk8A/DVVxjXm4ZEBJwhg=="
                    crossorigin="anonymous"
                    referrerpolicy="no-referrer"
                ></script>
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
