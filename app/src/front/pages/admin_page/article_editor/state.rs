use leptos::prelude::*;

use super::{save_article_action, publish_article_action, ArticleEditData, SaveArticleInput};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Split,
    Editor,
    Preview,
}

/// 記事編集フォームの状態管理
#[derive(Clone, Copy)]
pub struct ArticleFormState {
    pub title: RwSignal<String>,
    pub slug: RwSignal<String>,
    pub body: RwSignal<String>,
    pub description: RwSignal<String>,
    pub view_mode: RwSignal<ViewMode>,
    pub saving: RwSignal<bool>,
    pub publishing: RwSignal<bool>,
    pub message: RwSignal<Option<(bool, String)>>,
}

impl Default for ArticleFormState {
    fn default() -> Self {
        Self {
            title: RwSignal::new(String::new()),
            slug: RwSignal::new(String::new()),
            body: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            view_mode: RwSignal::new(ViewMode::default()),
            saving: RwSignal::new(false),
            publishing: RwSignal::new(false),
            message: RwSignal::new(None),
        }
    }
}

impl ArticleFormState {
    /// 記事データでフォームを初期化
    pub fn populate(&self, article: &ArticleEditData) {
        self.title.set(article.title.clone());
        self.slug.set(article.slug.clone());
        self.body.set(article.body.clone());
        self.description.set(article.description.clone().unwrap_or_default());
    }

    /// 操作中かどうか
    pub fn is_busy(&self) -> bool {
        self.saving.get() || self.publishing.get()
    }

    /// フォームデータをSaveArticleInputに変換
    fn to_save_input(&self, id: Option<String>) -> SaveArticleInput {
        let description = self.description.get();
        SaveArticleInput {
            id,
            title: self.title.get(),
            slug: self.slug.get(),
            body: self.body.get(),
            description: if description.is_empty() { None } else { Some(description) },
        }
    }

    /// 保存アクションを生成
    pub fn create_save_action<F>(&self, get_article_id: F) -> Action<(), ()>
    where
        F: Fn() -> Option<String> + Copy + Send + Sync + 'static,
    {
        let form = *self;
        Action::new(move |_: &()| {
            let input = form.to_save_input(get_article_id());
            async move {
                form.saving.set(true);
                form.message.set(None);

                let result = save_article_action(input).await;

                form.saving.set(false);

                match result {
                    Ok(_) => {
                        form.message.set(Some((true, "保存しました".to_string())));
                    }
                    Err(e) => {
                        form.message.set(Some((false, format!("エラー: {}", e))));
                    }
                }
            }
        })
    }

    /// 公開アクションを生成
    pub fn create_publish_action<F>(&self, get_article_id: F) -> Action<(), ()>
    where
        F: Fn() -> Option<String> + Copy + Send + Sync + 'static,
    {
        let form = *self;
        Action::new(move |_: &()| {
            let id = get_article_id();
            async move {
                let Some(id) = id else {
                    form.message.set(Some((false, "新規記事は先に保存してください".to_string())));
                    return;
                };

                form.publishing.set(true);
                form.message.set(None);

                let result = publish_article_action(id).await;

                form.publishing.set(false);

                match result {
                    Ok(_) => {
                        form.message.set(Some((true, "公開しました".to_string())));
                        let navigate = leptos_router::hooks::use_navigate();
                        navigate("/admin/articles", Default::default());
                    }
                    Err(e) => {
                        form.message.set(Some((false, format!("エラー: {}", e))));
                    }
                }
            }
        })
    }
}
