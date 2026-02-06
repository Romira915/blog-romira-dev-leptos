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
    // GCS / imgix (required for image upload)
    pub gcs_bucket: String,
    pub gcs_service_account_key_json: String,
    pub imgix_domain: String,    // e.g., "blog-romira.imgix.net"
    pub gcs_path_prefix: String, // e.g., "dev" or "prod"
    // Valkey (Redis-compatible) session store
    #[serde(default = "default_valkey_url")]
    pub valkey_url: String,
}

fn default_valkey_url() -> String {
    "redis://localhost:6379/0".to_string()
}

#[cfg(feature = "ssr")]
pub static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    envy::from_env().expect("Failed to read environment variables")
});
