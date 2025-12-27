pub mod article_editor;
mod article_editor_state;
pub mod article_list;
pub mod layout;

pub use article_editor::ArticleEditorPage;
pub use article_list::ArticleListPage;
pub use layout::AdminLayout;

// Re-export server functions and types
// Use cfg_attr to handle mutually exclusive features
#[cfg(all(feature = "ssr", not(feature = "hydrate")))]
pub use crate::server::auth::{get_auth_user, is_oauth_configured, AuthUser};

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
mod auth_stubs {
    use leptos::prelude::*;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct AuthUser {
        pub email: String,
        pub name: Option<String>,
        pub picture: Option<String>,
    }

    #[server(endpoint = "auth/me")]
    pub async fn get_auth_user() -> Result<Option<AuthUser>, ServerFnError> {
        unreachable!()
    }

    #[server(endpoint = "auth/configured")]
    pub async fn is_oauth_configured() -> Result<bool, ServerFnError> {
        unreachable!()
    }
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
pub use auth_stubs::{get_auth_user, is_oauth_configured, AuthUser};

// Fallback for --all-features (both enabled, used only for cargo check)
#[cfg(all(feature = "ssr", feature = "hydrate"))]
pub use crate::server::auth::{get_auth_user, is_oauth_configured, AuthUser};
