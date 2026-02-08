#![allow(dead_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use super::super::get_article_for_edit_handler;
use super::super::state::{ArticleFormState, ViewMode};
use super::MarkdownPreview;
use super::style;
use crate::common::imgix_url::{extract_base_url, generate_srcset};
use crate::constants::IMAGE_WIDTHS;
use crate::front::components::ImagePickerModal;
use crate::front::hooks::use_scroll_sync;
use crate::front::pages::admin_page::AdminLayout;

/// UTF-16コード単位位置をバイトインデックスに変換
fn utf16_offset_to_byte_index(s: &str, utf16_pos: usize) -> usize {
    s.char_indices()
        .scan(0usize, |utf16_count, (byte_idx, ch)| {
            if *utf16_count >= utf16_pos {
                return Some(Some(byte_idx));
            }
            *utf16_count += ch.len_utf16();
            Some(None)
        })
        .flatten()
        .next()
        .unwrap_or(s.len())
}

#[component]
pub fn ArticleEditorPage() -> impl IntoView {
    let params = use_params_map();
    let article_id = move || params.read().get("id").map(|s| s.to_string());
    let is_new = move || article_id().is_none();

    let form = ArticleFormState::default();

    // Load existing article if editing
    let article_resource = Resource::new(article_id, |id| async move {
        match id {
            Some(id) => get_article_for_edit_handler(id).await,
            None => Ok(None),
        }
    });

    // Populate form when article is loaded
    Effect::new(move || {
        if let Some(Ok(Some(article))) = article_resource.get() {
            form.populate(&article);
        }
    });

    // Actions
    let save_article = form.create_save_action(article_id);
    let publish_article = form.create_publish_action(article_id);

    // Scroll sync refs
    let editor_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let preview_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    // Image picker modals
    let show_cover_picker = RwSignal::new(false);
    let show_insert_picker = RwSignal::new(false);

    // Cover image selection callback
    let on_cover_select = Callback::new(
        move |image: crate::common::handlers::admin::images::ImageDto| {
            form.cover_image_url.set(Some(image.imgix_url));
        },
    );

    // Insert image callback
    let on_insert_select = Callback::new(
        move |image: crate::common::handlers::admin::images::ImageDto| {
            #[cfg(feature = "hydrate")]
            {
                if let Some(textarea) = editor_ref.get() {
                    let el: &web_sys::HtmlTextAreaElement = textarea.as_ref();
                    // selection_start(): Result<Option<u32>, JsValue> → 取得失敗時は先頭(0)扱い
                    let utf16_pos = el.selection_start().ok().flatten().unwrap_or(0) as usize;
                    let current_body = form.body.get();
                    let byte_index = utf16_offset_to_byte_index(&current_body, utf16_pos);

                    let image_url =
                        format!("{}?w={}&auto=format&q=75", image.imgix_url, IMAGE_WIDTHS[1]);
                    let srcset = generate_srcset(extract_base_url(&image.imgix_url), &IMAGE_WIDTHS);
                    let sizes = "(max-width: 800px) 100vw, 800px";
                    let (w, h) = match (image.width, image.height) {
                        (Some(w), Some(h)) => (w, h),
                        _ => (IMAGE_WIDTHS[1] as i32, 0),
                    };
                    let markdown_image = format!(
                        r#"<img src="{}" srcset="{}" sizes="{}" width="{}" height="{}" loading="lazy" alt="{}">"#,
                        image_url, srcset, sizes, w, h, image.filename,
                    );
                    let new_body = format!(
                        "{}{}{}",
                        &current_body[..byte_index],
                        markdown_image,
                        &current_body[byte_index..]
                    );

                    // 挿入テキスト分だけカーソル位置を進める（UTF-16単位で計算）
                    let new_cursor_utf16 =
                        utf16_pos + markdown_image.chars().map(|c| c.len_utf16()).sum::<usize>();
                    form.body.set(new_body);

                    // body更新後にDOMが再描画されるので、次のマイクロタスクでカーソル位置を復元
                    let el_clone = el.clone();
                    leptos::task::spawn_local(async move {
                        let _ = el_clone.set_selection_start(Some(new_cursor_utf16 as u32));
                        let _ = el_clone.set_selection_end(Some(new_cursor_utf16 as u32));
                    });
                }
            }
        },
    );

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
                                                if is_new() {
                                                    "新規作成"
                                                } else if form.is_draft.get() {
                                                    "下書き編集"
                                                } else {
                                                    "公開記事編集"
                                                }
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
                                                    } else if form.is_draft.get() {
                                                        "下書き保存"
                                                    } else {
                                                        "保存"
                                                    }
                                                }}
                                            </button>
                                            <Show when=move || !is_new() && form.is_draft.get()>
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

                                        <div class=style::form_row>
                                            <label class=style::label>"カバー画像"</label>
                                            <div class=style::cover_image_section>
                                                {move || {
                                                    form.cover_image_url
                                                        .get()
                                                        .map(|url| {
                                                            let preview_url = format!(
                                                                "{}?w={}&auto=format",
                                                                url,
                                                                IMAGE_WIDTHS[0],
                                                            );
                                                            view! {
                                                                <div class=style::cover_image_preview>
                                                                    <img src=preview_url alt="カバー画像" />
                                                                </div>
                                                            }
                                                        })
                                                }} <div class=style::cover_image_actions>
                                                    <button
                                                        type="button"
                                                        class=style::cover_select_button
                                                        on:click=move |_| show_cover_picker.set(true)
                                                    >
                                                        {move || {
                                                            if form.cover_image_url.get().is_some() {
                                                                "変更"
                                                            } else {
                                                                "選択"
                                                            }
                                                        }}
                                                    </button>
                                                    <Show when=move || { form.cover_image_url.get().is_some() }>
                                                        <button
                                                            type="button"
                                                            class=style::cover_remove_button
                                                            on:click=move |_| { form.cover_image_url.set(None) }
                                                        >
                                                            "解除"
                                                        </button>
                                                    </Show>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class=style::editor_toolbar>
                                        <div class=style::editor_tools>
                                            <button
                                                type="button"
                                                class=style::insert_image_button
                                                on:click=move |_| show_insert_picker.set(true)
                                            >
                                                "画像挿入"
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
                                        <Show when=move || {
                                            form.view_mode.get() != ViewMode::Preview
                                        }>
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
                                }
                            })
                    }}
                </Suspense>
                <ImagePickerModal show=show_cover_picker on_select=on_cover_select />
                <ImagePickerModal show=show_insert_picker on_select=on_insert_select />
            </div>
        </AdminLayout>
    }
}
