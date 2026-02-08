#![recursion_limit = "256"]

pub mod common;
pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod front;
#[cfg(feature = "ssr")]
pub(crate) mod server;

pub use front::app::{App, shell};
#[cfg(feature = "ssr")]
pub use server::{
    admin_routes::admin_routes,
    auth::{auth_routes, require_admin_auth},
    config::SERVER_CONFIG,
    contexts::AppState,
};
