use crate::common::dto::HomePageAuthorDto;
use crate::front::components::sns_icon::{GitHubIcon, XIcon};
use leptos::prelude::*;
use stylance::import_style;

import_style!(pub(crate) author_card_style, "author_card.module.scss");

#[component]
pub(crate) fn AuthorCard(
    author: HomePageAuthorDto,
    github_url: &'static str,
    x_url: &'static str,
) -> impl IntoView {
    view! {
        <section class=author_card_style::author_card>
            <div class=author_card_style::author_info>
                <img
                    src=author.avatar_url.get()
                    alt=format!("{}'s avatar", author.name.get())
                    width=100
                    height=100
                    class=author_card_style::author_avatar
                    loading="lazy"
                />
                <div class=author_card_style::author_description_container>
                    <h2 class=author_card_style::author_name>{author.name.get()}</h2>
                    <p class=author_card_style::author_description>{author.description.get()}</p>
                </div>
                <div class=author_card_style::author_sns>
                    <a href=github_url target="_blank" rel="noopener noreferrer">
                        <GitHubIcon />
                    </a>
                    <a href=x_url target="_blank" rel="noopener noreferrer">
                        <XIcon />
                    </a>
                </div>
            </div>
        </section>
    }
}
