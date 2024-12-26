use leptos::prelude::*;
use leptos_meta::{Meta, Title};

stylance::import_style!(pub my_style, "home_page.module.scss");

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    tracing::info!("HomePage");
    let number = Resource::new(|| (), |_| async move { get_number().await.unwrap() });

    view! {
        <HomePageMeta/>
        <h1>"HomePage"</h1>
        <p>"This is the home page."</p>
        <Suspense fallback=|| "Loading...">
            <p>"Get number: " {Suspend::new(async move {
            number.await
        })}</p>
        </Suspense>
    }
}

#[component]
pub(crate) fn HomePageMeta() -> impl IntoView {
    view! {
        <Title text="Romira's develop blog"/>
        <Meta property="og:title" content="Romira's develop blog"/>
        <Meta property="og:description" content="Rustaceanによる開発ブログです．技術共有や個人開発の進捗などを発信します．"/>
        <Meta property="og:type" content="article"/>
        <Meta property="og:url" content="https://blog.romira.dev"/>
        <Meta property="og:site_name" content="Romira's develop blog"/>
        <Meta property="og:image" content="https://blog-romira.imgix.net/46cea3d7-14ce-45bf-9d1e-52d1df39f2d2/romira'sdevelopblog.png"/>
        <Meta name="twitter:creator" content="@Romira915"/>
    }
}

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}
