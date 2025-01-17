use crate::common::dto::ArticleDetailDto;
use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) article_detail_style, "article_detail.module.scss");

#[component]
pub(crate) fn ArticleDetail(article: ArticleDetailDto) -> impl IntoView {
    view! {
        <article>
            <div>
                <h1>{article.title}</h1>
                <p>公開日: {article.published_at}</p>
                <p>カテゴリ: <span>{article.category}</span></p>
            </div>
            <section class=article_detail_style::markdown_body inner_html=article.body></section>
        </article>
    }
}
