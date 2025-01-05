pub(crate) mod common;
pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod front;
#[cfg(feature = "ssr")]
pub(crate) mod server;

pub use front::app::{shell, App};
#[cfg(feature = "ssr")]
pub use server::{config::SERVER_CONFIG, contexts::AppState};
