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
