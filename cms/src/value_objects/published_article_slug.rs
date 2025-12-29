use crate::error::CmsError;

/// 記事スラッグ（公開時は必須）
#[derive(Debug, Clone)]
pub struct PublishedArticleSlug(String);

impl PublishedArticleSlug {
    pub fn new(value: String) -> Result<Self, CmsError> {
        if value.trim().is_empty() {
            return Err(CmsError::ValidationError("スラッグは必須です".to_string()));
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
    fn test_有効なスラッグで作成できること() {
        let slug = PublishedArticleSlug::new("test-slug".to_string());
        assert!(slug.is_ok());
        assert_eq!(slug.unwrap().as_str(), "test-slug");
    }

    #[test]
    fn test_空文字列スラッグはエラーになること() {
        let slug = PublishedArticleSlug::new("".to_string());
        assert!(slug.is_err());
    }

    #[test]
    fn test_空白のみスラッグはエラーになること() {
        let slug = PublishedArticleSlug::new("   ".to_string());
        assert!(slug.is_err());
    }
}
