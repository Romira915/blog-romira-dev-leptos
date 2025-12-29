mod admin_article;
mod draft_article;
mod published_article;

pub use admin_article::AdminArticleService;
pub use draft_article::DraftArticleService;
pub use published_article::PublishedArticleService;

use chrono::{NaiveDateTime, Utc};

/// 現在時刻をUTC NaiveDateTimeで取得
fn utc_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}
