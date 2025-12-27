mod state;
mod view;

pub use view::ArticleEditorPage;

// Re-export from common handlers
pub use crate::common::handlers::admin::{
    ArticleEditData, SaveArticleInput, fetch_article_for_edit, publish_article_action,
    save_article_action,
};
