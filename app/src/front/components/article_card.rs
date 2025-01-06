use crate::common::dto::HomePageArticleDto;
use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) article_card_style, "article_card.module.scss");

#[component]
pub(crate) fn ArticleCard(article: ReadSignal<HomePageArticleDto>) -> impl IntoView {
    view! {
        <div class={article_card_style::article_card}>
            <a href={article.read().src.clone()} class={article_card_style::article_link}>
                <img src={article.read().thumbnail_url.clone()} alt="Thumbnail" class={article_card_style::article_thumbnail} />
                <div class={article_card_style::article_info}>
                    <h3 class={article_card_style::article_title}>{article.read().title.clone()}</h3>
                    <p class={article_card_style::article_category}>{article.read().category.clone()}</p>
                    <p class={article_card_style::article_published_at}>{article.read().published_at.to_string()}</p>
                </div>
            </a>
        </div>
    }
}
