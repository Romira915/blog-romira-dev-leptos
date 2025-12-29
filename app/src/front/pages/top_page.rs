mod top_page_meta;
mod top_page_view;

pub(crate) use top_page_meta::TopPageMeta;
pub(crate) use top_page_view::TopPage;

stylance::import_style!(pub(super) top_page_style, "top_page.module.scss");
