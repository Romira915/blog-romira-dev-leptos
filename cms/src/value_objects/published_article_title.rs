use crate::error::CmsError;

/// 記事タイトル（公開時は必須）
#[derive(Debug, Clone)]
pub struct PublishedArticleTitle(String);

impl PublishedArticleTitle {
    pub fn new(value: String) -> Result<Self, CmsError> {
        const MAX_LENGTH: usize = 200;

        if value.trim().is_empty() {
            return Err(CmsError::ValidationError("タイトルは必須です".to_string()));
        }
        if value.chars().count() > MAX_LENGTH {
            return Err(CmsError::ValidationError(format!(
                "タイトルは{}文字以内で入力してください",
                MAX_LENGTH
            )));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_有効なタイトルで作成できること() {
        let title = PublishedArticleTitle::new("テスト記事".to_string());
        assert!(title.is_ok());
        assert_eq!(title.unwrap().as_str(), "テスト記事");
    }

    #[test]
    fn test_空文字列はエラーになること() {
        let title = PublishedArticleTitle::new("".to_string());
        assert!(title.is_err());
    }

    #[test]
    fn test_空白のみはエラーになること() {
        let title = PublishedArticleTitle::new("   ".to_string());
        assert!(title.is_err());
    }

    #[test]
    fn test_200文字ちょうどは有効() {
        let title = PublishedArticleTitle::new("あ".repeat(200));
        assert!(title.is_ok());
    }

    #[test]
    fn test_201文字はエラーになること() {
        let title = PublishedArticleTitle::new("あ".repeat(201));
        assert!(title.is_err());
    }
}
