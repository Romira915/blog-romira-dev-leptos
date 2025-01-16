use select::document::Document;
use select::predicate::{Attr, Name, Predicate};
use tracing::instrument;

#[instrument]
pub(crate) async fn get_og_image_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<Option<String>, reqwest::Error> {
    let start = std::time::Instant::now();
    let response = client.get(url).send().await?;
    tracing::info!("client.get(url).send().await?: {} ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    let document = Document::from(response.text().await?.as_str());
    tracing::info!("Document::from(response.text().await?.as_str()): {} ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    if let Some(meta_tag) = document
        .find(Name("meta").and(Attr("property", "og:image")))
        .next()
    {
        if let Some(content) = meta_tag.attr("content") {
            return Ok(Some(content.to_string()));
        }
    }
    tracing::info!("if let Some(meta_tag) = document.find(Name(\"meta\").and(Attr(\"property\", \"og:image\"))).next() {{}}: {} ms", start.elapsed().as_millis());

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::server::models::newt_article::NewtArticleCollection;
    use crate::SERVER_CONFIG;
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
        assert_eq!(result.unwrap(), Some("https://example.com/image.jpg".to_string()));
    }
}
