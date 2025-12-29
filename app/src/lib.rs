pub mod common;
pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod front;
#[cfg(feature = "ssr")]
pub(crate) mod server;

pub use front::app::{App, shell};
#[cfg(feature = "ssr")]
pub use server::{auth::auth_routes, config::SERVER_CONFIG, contexts::AppState};
