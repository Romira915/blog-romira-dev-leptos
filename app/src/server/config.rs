use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize, Default, Debug)]
pub struct ServerConfig {
    pub host_name: String,
    pub new_relic_license_key: String,
    pub newt_cdn_api_token: String,
    pub newt_api_token: String,
    pub qiita_api_token: String,
    pub database_url: String,
    // OAuth (optional - required only for admin authentication)
    #[serde(default)]
    pub google_client_id: Option<String>,
    #[serde(default)]
    pub google_client_secret: Option<String>,
    #[serde(default)]
    pub app_url: Option<String>, // e.g., "http://localhost:3000" or "https://blog.romira.dev"
}

#[cfg(feature = "ssr")]
pub static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    envy::from_env().expect("Failed to read environment variables")
});
