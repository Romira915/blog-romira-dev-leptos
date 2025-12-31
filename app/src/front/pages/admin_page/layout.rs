use leptos::prelude::*;
use leptos_router::components::A;
use stylance::import_style;

use crate::front::pages::admin_page::{get_auth_user, is_oauth_configured};

import_style!(style, "layout.module.scss");

#[component]
pub fn AdminLayout(children: Children) -> impl IntoView {
    let auth_user = OnceResource::new(get_auth_user());
    let oauth_configured = OnceResource::new(is_oauth_configured());

    // features Cookie管理（クライアントサイドのみ）
    let (is_local, set_is_local) = signal(false);

    // クライアントサイドでCookieを読み取って初期化
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use leptos::wasm_bindgen::JsCast;
            use leptos::web_sys::HtmlDocument;
            let doc = leptos::prelude::document();
            if let Some(html_doc) = doc.dyn_ref::<HtmlDocument>() {
                let cookie = html_doc.cookie().unwrap_or_default();
                let has_local = cookie
                    .split(';')
                    .any(|part| part.trim() == "features=local");
                set_is_local.set(has_local);
            }
        }
    });

    let toggle_local = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use leptos::wasm_bindgen::JsCast;
            use leptos::web_sys::HtmlDocument;
            let doc = leptos::prelude::document();
            if let Some(html_doc) = doc.dyn_ref::<HtmlDocument>() {
                if is_local.get() {
                    // Cookieを削除
                    let _ = html_doc.set_cookie("features=; path=/; max-age=0");
                    set_is_local.set(false);
                } else {
                    // Cookieを設定
                    let _ = html_doc.set_cookie("features=local; path=/");
                    set_is_local.set(true);
                }
            }
        }
    };

    // Store children to use multiple times
    let children_view = children();

    view! {
        <div class=style::admin_layout>
            <nav class=style::sidebar>
                <div class=style::logo>
                    <A href="/">"Blog Admin"</A>
                </div>
                <ul class=style::nav_list>
                    <li>
                        <A href="/admin/articles" attr:class=style::nav_link>
                            "記事一覧"
                        </A>
                    </li>
                    <li>
                        <a href="/admin/articles/new" rel="external" class=style::nav_link>
                            "新規作成"
                        </a>
                    </li>
                </ul>
                <div class=style::features_toggle>
                    <label class=style::toggle_label>
                        <input
                            type="checkbox"
                            checked=is_local
                            on:change=toggle_local
                            class=style::toggle_checkbox
                        />
                        <span class=style::toggle_text>"ローカル記事表示"</span>
                    </label>
                </div>
                <div class=style::auth_section>
                    <Suspense fallback=|| ()>
                        {move || {
                            let configured = oauth_configured
                                .get()
                                .and_then(|r| r.ok())
                                .unwrap_or(false);
                            let user = auth_user.get().and_then(|r| r.ok()).flatten();
                            if !configured {

                                view! {
                                    <div class=style::auth_info>
                                        <span class=style::no_auth>"認証なし"</span>
                                    </div>
                                }
                                    .into_any()
                            } else if let Some(user) = user {
                                view! {
                                    <div class=style::auth_info>
                                        <span class=style::user_email>{user.email.clone()}</span>
                                        <a href="/auth/logout" class=style::logout_link>
                                            "ログアウト"
                                        </a>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class=style::auth_info>
                                        <a href="/auth/google" class=style::login_button>
                                            "Googleでログイン"
                                        </a>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </nav>
            <div class=style::main_content>
                {children_view}
                // Auth guard overlay - shows login prompt if OAuth configured but not logged in
                <Suspense fallback=|| ()>
                    {move || {
                        let configured = oauth_configured
                            .get()
                            .and_then(|r| r.ok())
                            .unwrap_or(false);
                        let user = auth_user.get().and_then(|r| r.ok()).flatten();
                        if configured && user.is_none() {

                            view! {
                                <div class=style::login_overlay>
                                    <div class=style::login_required>
                                        <h2>"ログインが必要です"</h2>
                                        <p>
                                            "管理画面にアクセスするにはGoogleアカウントでログインしてください。"
                                        </p>
                                        <a href="/auth/google" class=style::login_button_large>
                                            "Googleでログイン"
                                        </a>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
