use blog_romira_dev_app::App;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    #[cfg(debug_assertions)]
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(App);
}
