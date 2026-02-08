#![allow(dead_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use super::super::get_article_for_edit_handler;
use super::super::state::ArticleFormState;
use super::style;
use super::{ArticleForm, EditorHeader, EditorWorkspace};
use crate::common::imgix_url::{extract_base_url, generate_srcset};
use crate::constants::IMAGE_WIDTHS;
use crate::front::components::ImagePickerModal;
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
    let is_new = Signal::derive(move || article_id().is_none());

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
                            .map(|_: Result<_, _>| {
                                view! {
                                    <Show when=move || !form.is_fullscreen.get()>
                                        <EditorHeader
                                            form
                                            is_new
                                            save_action=save_article
                                            publish_action=publish_article
                                        />
                                        <ArticleForm form show_cover_picker />
                                    </Show>
                                    <EditorWorkspace
                                        form
                                        show_insert_picker
                                        editor_ref
                                        preview_ref
                                    />
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
