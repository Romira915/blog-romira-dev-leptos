use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize, Default, Debug)]
pub struct ServerConfig {
    pub new_relic_license_key: String,
    pub newt_cdn_api_token: String,
    pub newt_api_token: String,
    pub qiita_api_token: String,
}

#[cfg(feature = "ssr")]
pub static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(|| {
    dotenv::dotenv().ok();
    envy::from_env().expect("Failed to read environment variables")
});
