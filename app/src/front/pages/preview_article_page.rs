mod preview_article_page_meta;
mod preview_article_page_view;

pub(crate) use preview_article_page_meta::PreviewArticlePageMeta;
pub(crate) use preview_article_page_view::PreviewArticlePage;

use stylance::import_style;

import_style!(pub(crate) article_page_style, "article_page.module.scss");
