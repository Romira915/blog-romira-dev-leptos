use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use tracing::instrument;

#[instrument]
pub(crate) async fn get_og_image_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<Option<String>, reqwest::Error> {
    let response = client.get(url).send().await?;

    let document = Document::from(response.text().await?.as_str());

    if let Some(meta_tag) = document
        .find(Name("meta").and(Attr("property", "og:image")))
        .next()
    {
        if let Some(content) = meta_tag.attr("content") {
            return Ok(Some(content.to_string()));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_og_image_url() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock("GET", "/")
            .with_header("content-type", "text/html")
            .with_body(
                r#"
                    <html>
                        <head>
                            <meta property="og:image" content="https://example.com/image.jpg">
                        </head>
                    </html>
                "#,
            )
            .create();

        let client = reqwest::Client::new();
        let result = get_og_image_url(&client, &url).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some("https://example.com/image.jpg".to_string())
        );
    }
}
