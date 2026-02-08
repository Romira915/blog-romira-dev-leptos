use leptos::prelude::*;

use super::style;
use crate::constants::IMAGE_WIDTHS;
use crate::front::pages::admin_page::article_editor::state::ArticleFormState;

#[component]
pub fn ArticleForm(form: ArticleFormState, show_cover_picker: RwSignal<bool>) -> impl IntoView {
    view! {
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
                    on:input=move |ev| form.description.set(event_target_value(&ev))
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
    }
}
