use crate::common::dto::HomePageArticleDto;
use leptos::prelude::*;

use super::ArticleCard;
use super::article_card_style;

#[component]
pub(crate) fn ArticleCardList(
    #[prop(optional)] class: &'static str,
    articles: Vec<HomePageArticleDto>,
) -> impl IntoView {
    view! {
        <section class=format!(
            "{} {}",
            article_card_style::article_card_list,
            class,
        )>
            {articles
                .iter()
                .map(|article| {
                    view! { <ArticleCard article=article.clone() /> }
                })
                .collect_view()}
        </section>
    }
}
