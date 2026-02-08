#![allow(dead_code)]

use crate::common::dto::ArticleDetailDto;
use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) article_detail_style, "article_detail.module.scss");
import_style!(pub(crate) article_body_style, "article_body.module.scss");

#[component]
pub(crate) fn ArticleDetail(article: ArticleDetailDto) -> impl IntoView {
    view! {
        <article class=article_detail_style::article_detail_style>
            <h1 class=article_detail_style::article_title>{article.title}</h1>
            <div class=article_detail_style::article_meta>
                <ul class=article_detail_style::article_meta_category_list>
                    {article
                        .category
                        .into_iter()
                        .map(|category| {
                            view! {
                                <li class=article_detail_style::article_meta_category>
                                    {move || category.get()}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
                <p class=article_detail_style::article_meta_published_at>
                    // FIXME: まじめにやる
                    <time datetime=article
                        .first_published_at
                        .read()
                        .replace("年", "-")
                        .replace("月", "-")
                        .replace("日", "")>{article.first_published_at}</time>
                </p>
            </div>
            <figure class=article_detail_style::article_cover>
                <img
                    src=article.cover_image_url
                    srcset=article.cover_image_srcset
                    sizes="(max-width: 800px) 800px, (max-width: 1200px) 1200px, 1920px"
                    width=745
                    height=419
                    loading="lazy"
                    alt=format!("Cover image of {}", article.title.read())
                    class=article_detail_style::article_cover_image
                />
            </figure>
            <section
                class=format!(
                    "{} {}",
                    article_body_style::markdown_body,
                    article_detail_style::article_body,
                )
                inner_html=article.body
            ></section>
        </article>
    }
}
