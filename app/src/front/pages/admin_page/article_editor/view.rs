#![allow(dead_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use stylance::import_style;

use super::fetch_article_for_edit;
use super::state::{ArticleFormState, ViewMode};
use crate::front::pages::admin_page::AdminLayout;

import_style!(style, "article_editor.module.scss");

#[component]
pub fn ArticleEditorPage() -> impl IntoView {
    let params = use_params_map();
    let article_id = move || params.read().get("id").map(|s| s.to_string());
    let is_new = move || article_id().is_none();

    let form = ArticleFormState::default();

    // Load existing article if editing
    let article_resource = Resource::new(
        article_id,
        |id| async move {
            match id {
                Some(id) => fetch_article_for_edit(id).await,
                None => Ok(None),
            }
        },
    );

    // Populate form when article is loaded
    Effect::new(move || {
        if let Some(Ok(Some(article))) = article_resource.get() {
            form.populate(&article);
        }
    });

    // Actions
    let save_article = form.create_save_action(article_id);
    let publish_article = form.create_publish_action(article_id);

    view! {
        <AdminLayout>
            <div class=style::container>
                <Suspense fallback=move || {
                    view! { <p>"読み込み中..."</p> }
                }>
                    {move || {
                        article_resource
                            .get()
                            .map(|_| {
                                view! {
                                    <header class=style::header>
                                        <h1>
                                            {move || {
                                                if is_new() { "新規作成" } else { "下書き編集" }
                                            }}
                                        </h1>
                                        <div class=style::actions>
                                            <button
                                                class=style::save_button
                                                disabled=move || form.is_busy()
                                                on:click=move |_| {
                                                    let _ = save_article.dispatch(());
                                                }
                                            >
                                                {move || {
                                                    if form.saving.get() {
                                                        "保存中..."
                                                    } else {
                                                        "下書き保存"
                                                    }
                                                }}
                                            </button>
                                            <Show when=move || !is_new()>
                                                <button
                                                    class=style::publish_button
                                                    disabled=move || form.is_busy()
                                                    on:click=move |_| {
                                                        let _ = publish_article.dispatch(());
                                                    }
                                                >
                                                    {move || {
                                                        if form.publishing.get() {
                                                            "公開中..."
                                                        } else {
                                                            "公開"
                                                        }
                                                    }}
                                                </button>
                                            </Show>
                                        </div>
                                    </header>

                                    {move || {
                                        form.message
                                            .get()
                                            .map(|(success, msg)| {
                                                view! {
                                                    <div class=if success {
                                                        style::message_success
                                                    } else {
                                                        style::message_error
                                                    }>{msg}</div>
                                                }
                                            })
                                    }}

                                    <div class=style::form_section>
                                        <div class=style::form_row>
                                            <label class=style::label>"タイトル"</label>
                                            <input
                                                type="text"
                                                class=style::input
                                                prop:value=move || form.title.get()
                                                on:input=move |ev| form.title.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class=style::form_row>
                                            <label class=style::label>"スラッグ"</label>
                                            <input
                                                type="text"
                                                class=style::input
                                                prop:value=move || form.slug.get()
                                                on:input=move |ev| form.slug.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class=style::form_row>
                                            <label class=style::label>"説明"</label>
                                            <input
                                                type="text"
                                                class=style::input
                                                prop:value=move || form.description.get()
                                                on:input=move |ev| {
                                                    form.description.set(event_target_value(&ev))
                                                }
                                            />
                                        </div>

                                    </div>

                                    <div class=style::editor_toolbar>
                                        <div class=style::view_mode_buttons>
                                            <button
                                                class=move || {
                                                    if form.view_mode.get() == ViewMode::Split {
                                                        style::mode_button_active
                                                    } else {
                                                        style::mode_button
                                                    }
                                                }
                                                on:click=move |_| form.view_mode.set(ViewMode::Split)
                                            >
                                                "Split"
                                            </button>
                                            <button
                                                class=move || {
                                                    if form.view_mode.get() == ViewMode::Editor {
                                                        style::mode_button_active
                                                    } else {
                                                        style::mode_button
                                                    }
                                                }
                                                on:click=move |_| form.view_mode.set(ViewMode::Editor)
                                            >
                                                "Editor"
                                            </button>
                                            <button
                                                class=move || {
                                                    if form.view_mode.get() == ViewMode::Preview {
                                                        style::mode_button_active
                                                    } else {
                                                        style::mode_button
                                                    }
                                                }
                                                on:click=move |_| form.view_mode.set(ViewMode::Preview)
                                            >
                                                "Preview"
                                            </button>
                                        </div>
                                    </div>

                                    <div
                                        class=style::editor_container
                                        class:split=move || form.view_mode.get() == ViewMode::Split
                                    >
                                        <Show when=move || {
                                            form.view_mode.get() != ViewMode::Preview
                                        }>
                                            <div class=style::editor_pane>
                                                <textarea
                                                    class=style::textarea
                                                    prop:value=move || form.body.get()
                                                    on:input=move |ev| form.body.set(event_target_value(&ev))
                                                    placeholder="Markdownで記事を書く..."
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || form.view_mode.get() != ViewMode::Editor>
                                            <div class=style::preview_pane>
                                                <MarkdownPreview content=form.body />
                                            </div>
                                        </Show>
                                    </div>
                                }
                            })
                    }}
                </Suspense>
            </div>
        </AdminLayout>
    }
}

#[component]
fn MarkdownPreview(content: RwSignal<String>) -> impl IntoView {
    // TODO: Use comrak-wasm for client-side markdown rendering
    view! {
        <div class=style::preview_content>
            <pre>{move || content.get()}</pre>
        </div>
    }
}
