use leptos::prelude::*;
use stylance::import_style;

use crate::common::handlers::admin::images::{ImageDto, get_images_handler};

import_style!(style, "image_picker_modal.module.scss");

#[component]
fn ImageCard(
    image: ImageDto,
    on_select: Callback<ImageDto>,
    show: RwSignal<bool>,
) -> impl IntoView {
    let thumbnail_url = format!("{}?w=200&h=200&fit=crop&auto=format", image.imgix_url);
    let filename = image.filename.clone();
    view! {
        <div
            class=style::image_card
            on:click=move |_| {
                on_select.run(image.clone());
                show.set(false);
            }
        >
            <div class=style::image_thumbnail>
                <img src=thumbnail_url alt=filename.clone() loading="lazy" />
            </div>
            <div class=style::image_name>{filename}</div>
        </div>
    }
}

#[component]
pub fn ImagePickerModal(show: RwSignal<bool>, on_select: Callback<ImageDto>) -> impl IntoView {
    let images = Resource::new(
        move || show.get(),
        |visible| async move {
            if visible {
                get_images_handler().await.ok()
            } else {
                None
            }
        },
    );

    let on_overlay_click = move |_| {
        show.set(false);
    };

    let on_modal_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    view! {
        <Show when=move || show.get()>
            <div class=style::overlay on:click=on_overlay_click>
                <div class=style::modal on:click=on_modal_click>
                    <div class=style::header>
                        <h2>"画像を選択"</h2>
                        <button class=style::close_button on:click=move |_| show.set(false)>
                            "×"
                        </button>
                    </div>
                    <div class=style::body>
                        <Suspense fallback=move || {
                            view! { <p class=style::loading>"画像を読み込み中..."</p> }
                        }>
                            {move || {
                                images
                                    .get()
                                    .map(|result| {
                                        match result {
                                            Some(list) if list.is_empty() => {
                                                view! {
                                                    <p class=style::empty>"まだ画像がありません"</p>
                                                }
                                                    .into_any()
                                            }
                                            Some(list) => {
                                                view! {
                                                    <div class=style::grid>
                                                        {list
                                                            .into_iter()
                                                            .map(|image| {
                                                                view! { <ImageCard image on_select show /> }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            None => {
                                                view! {
                                                    <p class=style::error>
                                                        "画像の読み込みに失敗しました"
                                                    </p>
                                                }
                                                    .into_any()
                                            }
                                        }
                                    })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </Show>
    }
}
