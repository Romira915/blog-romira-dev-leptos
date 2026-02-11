use leptos::prelude::*;

use super::super::get_categories_handler;
use super::style;
use crate::constants::IMAGE_WIDTHS;
use crate::front::pages::admin_page::article_editor::state::ArticleFormState;

#[component]
pub fn ArticleForm(form: ArticleFormState, show_cover_picker: RwSignal<bool>) -> impl IntoView {
    let tag_input = RwSignal::new(String::new());

    // 既存カテゴリ候補を取得
    let categories_resource = Resource::new(
        || (),
        |_| async move { get_categories_handler().await.unwrap_or_default() },
    );

    let on_tag_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            let value = tag_input.get().trim().to_string();
            if !value.is_empty() {
                let mut current = form.categories.get();
                if !current.iter().any(|c| c.eq_ignore_ascii_case(&value)) {
                    current.push(value);
                    form.categories.set(current);
                }
                tag_input.set(String::new());
            }
        }
    };

    let remove_tag = move |index: usize| {
        let mut current = form.categories.get();
        if index < current.len() {
            current.remove(index);
            form.categories.set(current);
        }
    };

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
                <label class=style::label>"タグ"</label>
                <div class=style::tag_chips>
                    <For
                        each=move || {
                            form.categories.get().into_iter().enumerate().collect::<Vec<_>>()
                        }
                        key=|(_i, name)| name.clone()
                        let:item
                    >
                        {
                            let (index, name) = item;
                            view! {
                                <span class=style::tag_chip>
                                    {name}
                                    <button
                                        type="button"
                                        class=style::tag_chip_remove
                                        on:click=move |_| remove_tag(index)
                                    >
                                        "\u{00d7}"
                                    </button>
                                </span>
                            }
                        }
                    </For>
                </div>
                <input
                    type="text"
                    class=style::input
                    list="category-suggestions"
                    placeholder="タグを入力してEnterで追加"
                    prop:value=move || tag_input.get()
                    on:input=move |ev| tag_input.set(event_target_value(&ev))
                    on:keydown=on_tag_keydown
                />
                <Suspense fallback=|| ()>
                    {move || {
                        categories_resource
                            .get()
                            .map(|cats| {
                                view! {
                                    <datalist id="category-suggestions">
                                        {cats
                                            .into_iter()
                                            .map(|c| {
                                                view! { <option value=c /> }
                                            })
                                            .collect::<Vec<_>>()}
                                    </datalist>
                                }
                            })
                    }}
                </Suspense>
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
