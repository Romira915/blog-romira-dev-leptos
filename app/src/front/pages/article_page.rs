mod article_page_meta;
mod article_page_view;

pub(crate) use article_page_meta::ArticlePageMeta;
pub(crate) use article_page_view::ArticlePage;

use stylance::import_style;

import_style!(pub(crate) article_page_style, "article_page.module.scss");
