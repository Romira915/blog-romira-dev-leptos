/// imgix CDN URL生成サービス
#[derive(Clone, Debug)]
pub struct ImgixService {
    domain: String,
}

impl ImgixService {
    pub fn new(domain: String) -> Self {
        Self { domain }
    }

    /// GCSパスからimgix URLを生成
    pub fn generate_url(&self, gcs_path: &str) -> String {
        format!("https://{}/{}", self.domain, gcs_path)
    }

    /// GCSパスからimgix URL（サイズ指定付き）を生成
    pub fn generate_url_with_width(&self, gcs_path: &str, width: u32) -> String {
        format!(
            "https://{}/{}?w={}&auto=format",
            self.domain, gcs_path, width
        )
    }

    /// ドメインを取得
    pub fn domain(&self) -> &str {
        self.domain.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_url() {
        let service = ImgixService::new("blog-romira.imgix.net".to_string());

        assert_eq!(
            service.generate_url("images/test.jpg"),
            "https://blog-romira.imgix.net/images/test.jpg"
        );
    }

    #[test]
    fn test_generate_url_with_width() {
        let service = ImgixService::new("blog-romira.imgix.net".to_string());

        assert_eq!(
            service.generate_url_with_width("images/test.jpg", 800),
            "https://blog-romira.imgix.net/images/test.jpg?w=800&auto=format"
        );
    }
}
