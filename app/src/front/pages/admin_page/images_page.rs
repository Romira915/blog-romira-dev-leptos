use leptos::prelude::*;
use stylance::import_style;

use super::AdminLayout;
use crate::front::components::{ImageList, UploadArea};

import_style!(style, "images_page.module.scss");

#[component]
pub fn ImagesPage() -> impl IntoView {
    // アップロード完了時のリフレッシュトリガー
    let refresh_trigger = RwSignal::new(0u32);

    let on_upload_complete = move || {
        refresh_trigger.update(|n| *n += 1);
    };

    view! {
        <AdminLayout>
            <div class=style::container>
                <header class=style::header>
                    <h1>"画像管理"</h1>
                </header>
                <section class=style::upload_section>
                    <UploadArea on_upload_complete=on_upload_complete />
                </section>
                <section class=style::images_section>
                    <h2>"アップロード済み画像"</h2>
                    <ImageList refresh_trigger=refresh_trigger />
                </section>
            </div>
        </AdminLayout>
    }
}
