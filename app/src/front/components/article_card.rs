use crate::common::dto::HomePageArticleDto;
use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) article_card_style, "article_card.module.scss");

#[component]
pub(crate) fn ArticleCard(article: ReadSignal<HomePageArticleDto>) -> impl IntoView {
    view! {
        <article class=article_card_style::article_card>
            <a href=article.read().src.clone() class=article_card_style::article_link>
                <figure>
                    <img
                        src=article.read().thumbnail_url.clone()
                        alt="Article thumbnail for {article.read().title.clone()}"
                        class=article_card_style::article_thumbnail
                    />
                    <figcaption class=article_card_style::article_info>
                        <h2 class=article_card_style::article_title>
                            {article.read().title.clone()}
                        </h2>
                        <ul class=article_card_style::article_category_list>
                            <li class=article_card_style::article_category>
                                {article.read().category.clone()}
                            </li>
                        </ul>
                        <p class=article_card_style::article_published_at>
                            {article.read().published_at.to_string()}
                        </p>
                    </figcaption>
                </figure>
            </a>
        </article>
    }
}
