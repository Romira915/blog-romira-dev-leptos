use leptos::prelude::*;
use stylance::import_style;

use super::AdminLayout;

import_style!(style, "images_page.module.scss");

#[component]
pub fn ImagesPage() -> impl IntoView {
    view! {
        <AdminLayout>
            <div class=style::container>
                <header class=style::header>
                    <h1>"画像管理"</h1>
                </header>
            </div>
        </AdminLayout>
    }
}
