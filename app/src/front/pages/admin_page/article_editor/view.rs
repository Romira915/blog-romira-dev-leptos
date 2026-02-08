#![allow(dead_code)]

mod article_editor_page;
mod article_form;
mod editor_header;
mod editor_workspace;
mod markdown_preview;

pub use article_editor_page::ArticleEditorPage;

use article_form::ArticleForm;
use editor_header::EditorHeader;
use editor_workspace::EditorWorkspace;
use markdown_preview::MarkdownPreview;

use stylance::import_style;

import_style!(pub(super) style, "article_editor.module.scss");
