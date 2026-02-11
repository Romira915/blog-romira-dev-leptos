mod state;
mod view;

pub use view::ArticleEditorPage;

// Re-export from common handlers
pub use crate::common::handlers::admin::{
    ArticleEditData, DeleteArticleInput, PublishArticleInput, SaveDraftInput, SavePublishedInput,
    delete_article_handler, get_article_for_edit_handler, publish_article_handler,
    save_draft_handler, save_published_handler,
};
