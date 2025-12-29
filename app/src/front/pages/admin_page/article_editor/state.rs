use leptos::prelude::*;

use super::{
    ArticleEditData, PublishArticleInput, SaveDraftInput, SavePublishedInput,
    publish_article_handler, save_draft_handler, save_published_handler,
};

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
    pub is_draft: RwSignal<bool>,
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
            is_draft: RwSignal::new(true), // 新規作成時は下書き
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
        self.description
            .set(article.description.clone().unwrap_or_default());
        self.is_draft.set(article.is_draft);
    }

    /// 操作中かどうか
    pub fn is_busy(&self) -> bool {
        self.saving.get() || self.publishing.get()
    }

    /// 下書き保存用の入力データを生成
    fn as_draft_input(&self, id: Option<String>) -> SaveDraftInput {
        let description = self.description.get();
        SaveDraftInput {
            id,
            title: self.title.get(),
            slug: self.slug.get(),
            body: self.body.get(),
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
        }
    }

    /// 公開記事保存用の入力データを生成
    fn as_published_input(&self, id: String) -> SavePublishedInput {
        let description = self.description.get();
        SavePublishedInput {
            id,
            title: self.title.get(),
            slug: self.slug.get(),
            body: self.body.get(),
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
        }
    }

    /// 保存アクションを生成（下書き/公開で適切なエンドポイントを呼び分け）
    pub fn create_save_action<F>(&self, get_article_id: F) -> Action<(), ()>
    where
        F: Fn() -> Option<String> + Copy + Send + Sync + 'static,
    {
        let form = *self;
        Action::new(move |_: &()| {
            let id = get_article_id();
            let is_draft = form.is_draft.get();
            async move {
                form.saving.set(true);
                form.message.set(None);

                let result = if is_draft {
                    // 下書きの保存
                    let input = form.as_draft_input(id);
                    save_draft_handler(input).await
                } else {
                    // 公開記事の保存（IDが必須）
                    let Some(id) = id else {
                        form.saving.set(false);
                        form.message
                            .set(Some((false, "公開記事の保存にはIDが必要です".to_string())));
                        return;
                    };
                    let input = form.as_published_input(id);
                    save_published_handler(input).await
                };

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

    /// 公開アクションを生成（save → publish の順で実行）
    pub fn create_publish_action<F>(&self, get_article_id: F) -> Action<(), ()>
    where
        F: Fn() -> Option<String> + Copy + Send + Sync + 'static,
    {
        let form = *self;
        Action::new(move |_: &()| {
            let id = get_article_id();
            async move {
                let Some(id) = id else {
                    form.message
                        .set(Some((false, "新規記事は先に保存してください".to_string())));
                    return;
                };

                form.publishing.set(true);
                form.message.set(None);

                // まず下書きを保存
                let save_input = form.as_draft_input(Some(id.clone()));
                if let Err(e) = save_draft_handler(save_input).await {
                    form.publishing.set(false);
                    form.message
                        .set(Some((false, format!("保存エラー: {}", e))));
                    return;
                }

                // 次に公開
                let result = publish_article_handler(PublishArticleInput { id }).await;

                form.publishing.set(false);

                match result {
                    Ok(_) => {
                        form.message.set(Some((true, "公開しました".to_string())));
                        let navigate = leptos_router::hooks::use_navigate();
                        navigate("/admin/articles", Default::default());
                    }
                    Err(e) => {
                        form.message
                            .set(Some((false, format!("公開エラー: {}", e))));
                    }
                }
            }
        })
    }
}
