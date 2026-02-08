use leptos::prelude::*;

use super::style;
use crate::front::pages::admin_page::article_editor::state::ArticleFormState;

#[component]
pub fn EditorHeader(
    form: ArticleFormState,
    is_new: Signal<bool>,
    save_action: Action<(), ()>,
    publish_action: Action<(), ()>,
) -> impl IntoView {
    view! {
        <header class=style::header>
            <h1>
                {move || {
                    if is_new.get() {
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
                        let _ = save_action.dispatch(());
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
                <Show when=move || !is_new.get() && form.is_draft.get()>
                    <button
                        class=style::publish_button
                        disabled=move || form.is_busy()
                        on:click=move |_| {
                            let _ = publish_action.dispatch(());
                        }
                    >
                        {move || { if form.publishing.get() { "公開中..." } else { "公開" } }}
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
    }
}
