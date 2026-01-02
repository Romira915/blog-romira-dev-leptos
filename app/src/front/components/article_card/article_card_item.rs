use crate::common::dto::{ArticleSource, HomePageArticleDto};
use leptos::prelude::*;

use super::article_card_style;

#[component]
pub(crate) fn ArticleCard(article: HomePageArticleDto) -> impl IntoView {
    let (a_target, a_ref) = match article.article_source {
        ArticleSource::Newt | ArticleSource::Local => ("", ""),
        _ => ("_blank", "noopener noreferrer"),
    };

    view! {
        <article class=article_card_style::article_card>
            <a
                href=article.src.get()
                aria-label=article.title.get()
                class=article_card_style::article_link
                target=a_target
                rel=a_ref
            >
                <figure class=article_card_style::article_figure>
                    <img
                        src=article.thumbnail_url.get()
                        width=228
                        height=128
                        alt=format!("Thumbnail of {}", article.title.get())
                        class=article_card_style::article_thumbnail
                    />
                    <figcaption class=article_card_style::article_info>
                        <h2 class=article_card_style::article_title>{article.title.get()}</h2>
                        <ul class=article_card_style::article_category_list>
                            {article
                                .category
                                .iter()
                                .map(|category| {
                                    view! {
                                        <li class=article_card_style::article_category>
                                            {category.get()}
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                        <p class=article_card_style::article_published_at>
                            {article.first_published_at.get()}
                        </p>
                    </figcaption>
                </figure>
            </a>
        </article>
    }
}
