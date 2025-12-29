mod app_component;
mod shell;

pub use app_component::App;
pub use shell::shell;

#[cfg(debug_assertions)]
const ASSETS_ROOT: &str = "";
#[cfg(not(debug_assertions))]
const ASSETS_ROOT: &str = std::env!("ASSETS_ROOT");
