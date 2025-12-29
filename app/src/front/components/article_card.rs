mod article_card_item;
mod article_card_list;

pub(crate) use article_card_item::ArticleCard;
pub(crate) use article_card_list::ArticleCardList;

use stylance::import_style;

import_style!(pub(super) article_card_style, "article_card.module.scss");
