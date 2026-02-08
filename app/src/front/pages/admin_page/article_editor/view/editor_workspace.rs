use leptos::prelude::*;

use super::MarkdownPreview;
use super::style;
use crate::front::hooks::use_scroll_sync;
use crate::front::pages::admin_page::article_editor::state::{ArticleFormState, ViewMode};

#[component]
pub fn EditorWorkspace(
    form: ArticleFormState,
    show_insert_picker: RwSignal<bool>,
    editor_ref: NodeRef<leptos::html::Textarea>,
    preview_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    view! {
        <div
            class=move || { if form.is_fullscreen.get() { style::fullscreen } else { "" } }
            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Escape" && form.is_fullscreen.get() {
                    form.is_fullscreen.set(false);
                }
            }
        >
            <div class=style::editor_toolbar>
                <div class=style::editor_tools>
                    <button
                        type="button"
                        class=style::insert_image_button
                        on:click=move |_| show_insert_picker.set(true)
                    >
                        "画像挿入"
                    </button>
                    <button
                        type="button"
                        class=style::fullscreen_button
                        on:click=move |_| form.is_fullscreen.update(|v| *v = !*v)
                    >
                        {move || {
                            if form.is_fullscreen.get() { "全画面解除" } else { "全画面" }
                        }}
                    </button>
                </div>
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
                <Show when=move || form.view_mode.get() != ViewMode::Preview>
                    <div class=style::editor_pane>
                        <textarea
                            class=style::textarea
                            node_ref=editor_ref
                            prop:value=move || form.body.get()
                            on:input=move |ev| form.body.set(event_target_value(&ev))
                            on:scroll=move |_| use_scroll_sync(editor_ref, preview_ref)
                            placeholder="Markdownで記事を書く..."
                        />
                    </div>
                </Show>
                <Show when=move || form.view_mode.get() != ViewMode::Editor>
                    <div class=style::preview_pane node_ref=preview_ref>
                        <MarkdownPreview content=form.body />
                    </div>
                </Show>
            </div>
        </div>
    }
}
