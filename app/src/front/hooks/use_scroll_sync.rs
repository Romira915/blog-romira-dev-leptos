use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{HtmlElement, HtmlTextAreaElement};

/// エディターとプレビューのスクロール同期
pub fn use_scroll_sync(
    editor_ref: NodeRef<leptos::html::Textarea>,
    preview_ref: NodeRef<leptos::html::Div>,
) {
    if let (Some(editor), Some(preview)) = (editor_ref.get(), preview_ref.get()) {
        let editor_el = editor
            .dyn_ref::<HtmlTextAreaElement>()
            .expect("editor should be HtmlTextAreaElement");
        let preview_el = preview
            .dyn_ref::<HtmlElement>()
            .expect("preview should be HtmlElement");

        let editor_scroll_top = editor_el.scroll_top() as f64;
        let editor_scroll_height = editor_el.scroll_height() as f64;
        let editor_client_height = editor_el.client_height() as f64;

        let max_scroll = editor_scroll_height - editor_client_height;
        if max_scroll > 0.0 {
            let scroll_ratio = editor_scroll_top / max_scroll;

            let preview_scroll_height = preview_el.scroll_height() as f64;
            let preview_client_height = preview_el.client_height() as f64;
            let preview_max_scroll = preview_scroll_height - preview_client_height;

            let target_scroll = (scroll_ratio * preview_max_scroll) as i32;
            preview_el.set_scroll_top(target_scroll);
        }
    }
}
