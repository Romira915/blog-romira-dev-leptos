use tracing::instrument;
use url::Url;

#[instrument]
pub(crate) fn to_optimize_thumbnail_url(url: &str) -> String {
    let mut url = Url::parse(url).expect("Failed to parse URL");
    url.query_pairs_mut()
        .append_pair("fit", "crop")
        .append_pair("w", "940")
        .append_pair("h", "528")
        .append_pair("q", "75")
        .append_pair("auto", "format,compress,enhance");

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_optimize_thumbnail_url() {
        let url = "https://example.com/image.jpg";
        let expected = "https://example.com/image.jpg?fit=crop&w=940&h=528&q=75&auto=format%2Ccompress%2Cenhance";
        assert_eq!(to_optimize_thumbnail_url(url), expected);
    }
}
