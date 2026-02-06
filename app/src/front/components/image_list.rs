use leptos::prelude::*;
use stylance::import_style;

use crate::common::handlers::admin::images::{
    DeleteImageInput, ImageDto, delete_image_handler, get_images_handler,
};

import_style!(pub style, "image_list.module.scss");

#[component]
pub fn ImageList(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let images = Resource::new(move || refresh_trigger.get(), |_| get_images_handler());

    let deleting: RwSignal<Option<String>> = RwSignal::new(None);
    let message: RwSignal<Option<(bool, String)>> = RwSignal::new(None);

    view! {
        {move || {
            message
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
        <Suspense fallback=move || {
            view! { <p class=style::loading>"画像を読み込み中..."</p> }
        }>
            {move || {
                images
                    .get()
                    .map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! { <p class=style::empty>"まだ画像がありません"</p> }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <div class=style::grid>
                                        {list
                                            .into_iter()
                                            .map(|image| {
                                                view! {
                                                    <ImageCard
                                                        image=image
                                                        deleting=deleting
                                                        message=message
                                                        refresh_trigger=refresh_trigger
                                                    />
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                                    .into_any()
                            }
                            Err(e) => {
                                view! { <p class=style::error>{format!("エラー: {}", e)}</p> }
                                    .into_any()
                            }
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn ImageCard(
    image: ImageDto,
    deleting: RwSignal<Option<String>>,
    message: RwSignal<Option<(bool, String)>>,
    refresh_trigger: RwSignal<u32>,
) -> impl IntoView {
    let id = image.id.clone();
    let id_for_delete = id.clone();
    let id_for_check = id.clone();
    let id_for_check2 = id.clone();
    let url = image.imgix_url.clone();
    let thumbnail_url = format!("{}?w=200&h=200&fit=crop&auto=format", image.imgix_url);
    let size_display = format_file_size(image.size_bytes);
    let filename_for_alt = image.filename.clone();
    let filename_for_title = image.filename.clone();

    let is_deleting = move || deleting.get().as_ref() == Some(&id_for_check);
    let is_deleting2 = move || deleting.get().as_ref() == Some(&id_for_check2);

    let on_delete = move || {
        let id = id_for_delete.clone();
        leptos::task::spawn_local(async move {
            deleting.set(Some(id.clone()));
            message.set(None);

            match delete_image_handler(DeleteImageInput { id }).await {
                Ok(()) => {
                    message.set(Some((true, "画像を削除しました".to_string())));
                    refresh_trigger.update(|n| *n += 1);
                }
                Err(e) => {
                    message.set(Some((false, format!("削除エラー: {}", e))));
                }
            }

            deleting.set(None);
        });
    };

    let on_copy = move || {
        let url = url.clone();
        #[cfg(feature = "hydrate")]
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&url);
            message.set(Some((true, "URLをコピーしました".to_string())));
        }
    };

    view! {
        <div class=style::card>
            <div class=style::thumbnail>
                <img src=thumbnail_url alt=filename_for_alt loading="lazy" />
            </div>
            <div class=style::info>
                <p class=style::filename title=filename_for_title>
                    {image.filename}
                </p>
                <p class=style::meta>{size_display}</p>
            </div>
            <div class=style::actions>
                <button
                    class=style::copy_button
                    on:click=move |_| on_copy()
                    title="imgix URLをコピー"
                >
                    "📋"
                </button>
                <button
                    class=style::delete_button
                    on:click=move |_| {
                        #[cfg(feature = "hydrate")]
                        if web_sys::window()
                            .and_then(|w| {
                                w.confirm_with_message("この画像を削除しますか？").ok()
                            })
                            .unwrap_or(false)
                        {
                            on_delete();
                        }
                    }
                    disabled=is_deleting
                    title="削除"
                >
                    {move || if is_deleting2() { "..." } else { "🗑️" }}
                </button>
            </div>
        </div>
    }
}

fn format_file_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;

    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
