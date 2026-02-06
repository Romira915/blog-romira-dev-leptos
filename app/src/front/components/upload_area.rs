use leptos::prelude::*;
use stylance::import_style;

use crate::common::handlers::admin::images::{
    GenerateUploadUrlInput, RegisterImageInput, generate_upload_url_handler, register_image_handler,
};

import_style!(pub style, "upload_area.module.scss");

#[component]
pub fn UploadArea<F>(on_upload_complete: F) -> impl IntoView
where
    F: Fn() + Clone + Send + Sync + 'static,
{
    let uploading = RwSignal::new(false);
    let message: RwSignal<Option<(bool, String)>> = RwSignal::new(None);
    let dragging = RwSignal::new(false);
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_upload_complete = StoredValue::new(on_upload_complete);

    let upload_file =
        move |file_name: String, content_type: String, size: i64, file_data: Vec<u8>| {
            let on_complete = on_upload_complete.get_value();
            async move {
                uploading.set(true);
                message.set(None);

                // Step 1: Get signed upload URL
                let url_response = match generate_upload_url_handler(GenerateUploadUrlInput {
                    filename: file_name.clone(),
                    content_type: content_type.clone(),
                    size_bytes: size,
                })
                .await
                {
                    Ok(resp) => resp,
                    Err(e) => {
                        message.set(Some((false, format!("URL生成エラー: {}", e))));
                        uploading.set(false);
                        return;
                    }
                };

                // Step 2: Upload to GCS via signed URL
                #[cfg(feature = "hydrate")]
                {
                    use wasm_bindgen::JsCast;
                    use wasm_bindgen_futures::JsFuture;
                    use web_sys::{RequestInit, RequestMode};

                    let body = js_sys::Uint8Array::from(file_data.as_slice());

                    let opts = RequestInit::new();
                    opts.set_method("PUT");
                    opts.set_mode(RequestMode::Cors);
                    opts.set_body(&body.into());

                    let request = match web_sys::Request::new_with_str_and_init(
                        &url_response.upload_url,
                        &opts,
                    ) {
                        Ok(req) => req,
                        Err(e) => {
                            message.set(Some((false, format!("リクエスト作成エラー: {:?}", e))));
                            uploading.set(false);
                            return;
                        }
                    };

                    if let Err(e) = request.headers().set("Content-Type", &content_type) {
                        message.set(Some((false, format!("ヘッダー設定エラー: {:?}", e))));
                        uploading.set(false);
                        return;
                    }

                    let window = match web_sys::window() {
                        Some(w) => w,
                        None => {
                            message.set(Some((
                                false,
                                "windowオブジェクトが取得できません".to_string(),
                            )));
                            uploading.set(false);
                            return;
                        }
                    };

                    let resp_value = match JsFuture::from(window.fetch_with_request(&request)).await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            message.set(Some((false, format!("アップロードエラー: {:?}", e))));
                            uploading.set(false);
                            return;
                        }
                    };

                    let response: web_sys::Response = match resp_value.dyn_into() {
                        Ok(r) => r,
                        Err(_) => {
                            message.set(Some((false, "レスポンスの変換に失敗".to_string())));
                            uploading.set(false);
                            return;
                        }
                    };

                    if !response.ok() {
                        message.set(Some((
                            false,
                            format!("GCSアップロード失敗: {}", response.status()),
                        )));
                        uploading.set(false);
                        return;
                    }
                }

                // Step 3: Register image in DB
                match register_image_handler(RegisterImageInput {
                    filename: file_name,
                    gcs_path: url_response.gcs_path,
                    mime_type: content_type,
                    size_bytes: size,
                    width: None,
                    height: None,
                    alt_text: None,
                })
                .await
                {
                    Ok(_) => {
                        message.set(Some((true, "アップロード完了".to_string())));
                        on_complete();
                    }
                    Err(e) => {
                        message.set(Some((false, format!("DB登録エラー: {}", e))));
                    }
                }

                uploading.set(false);
            }
        };

    let handle_files = move |files: Vec<(String, String, i64, Vec<u8>)>| {
        for (name, content_type, size, data) in files {
            leptos::task::spawn_local(upload_file(name, content_type, size, data));
        }
    };

    let on_drop = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        dragging.set(false);

        #[cfg(feature = "hydrate")]
        if let Some(data_transfer) = ev.data_transfer()
            && let Some(files) = data_transfer.files()
        {
            process_file_list(files, handle_files);
        }
    };

    let on_dragover = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        dragging.set(true);
    };

    let on_dragleave = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        dragging.set(false);
    };

    let on_file_input_change = move |_ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        if let Some(input) = input_ref.get() {
            if let Some(files) = input.files() {
                process_file_list(files, handle_files);
            }
            // Reset input so the same file can be selected again
            input.set_value("");
        }
    };

    let on_click = move |_| {
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    view! {
        <div
            class=move || {
                if dragging.get() {
                    format!("{} {}", style::upload_area, style::dragging)
                } else {
                    style::upload_area.to_string()
                }
            }
            on:drop=on_drop
            on:dragover=on_dragover
            on:dragleave=on_dragleave
            on:click=on_click
        >
            <input
                type="file"
                accept="image/*"
                class=style::file_input
                node_ref=input_ref
                on:change=on_file_input_change
            />
            <div class=style::upload_content>
                <Show
                    when=move || uploading.get()
                    fallback=move || {
                        view! {
                            <div class=style::upload_icon>"📁"</div>
                            <p class=style::upload_text>
                                "ここにファイルをドロップ、またはクリックして選択"
                            </p>
                            <p class=style::upload_hint>
                                "対応形式: JPEG, PNG, GIF, WebP (最大10MB)"
                            </p>
                        }
                    }
                >
                    <div class=style::uploading_spinner></div>
                    <p class=style::upload_text>"アップロード中..."</p>
                </Show>
            </div>
        </div>
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
    }
}

#[cfg(feature = "hydrate")]
fn process_file_list<F>(files: web_sys::FileList, handler: F)
where
    F: Fn(Vec<(String, String, i64, Vec<u8>)>) + 'static,
{
    use wasm_bindgen::JsCast;

    let file_count = files.length();
    let results = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let pending = std::rc::Rc::new(std::cell::Cell::new(file_count));
    let handler = std::rc::Rc::new(handler);

    for i in 0..file_count {
        if let Some(file) = files.get(i) {
            let name = file.name();
            let content_type = file.type_();
            let size = file.size() as i64;

            let results = results.clone();
            let pending = pending.clone();
            let handler = handler.clone();

            let reader = match web_sys::FileReader::new() {
                Ok(r) => r,
                Err(_) => continue,
            };

            let reader_clone = reader.clone();
            let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Ok(result) = reader_clone.result()
                    && let Some(array_buffer) = result.dyn_ref::<js_sys::ArrayBuffer>()
                {
                    let uint8_array = js_sys::Uint8Array::new(array_buffer);
                    let data = uint8_array.to_vec();
                    results
                        .borrow_mut()
                        .push((name.clone(), content_type.clone(), size, data));
                }

                let remaining = pending.get() - 1;
                pending.set(remaining);

                if remaining == 0 {
                    let files_data = results.borrow().clone();
                    handler(files_data);
                }
            }) as Box<dyn FnMut(_)>);

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            let _ = reader.read_as_array_buffer(&file);
        }
    }
}
