/// Blob storage operations for the Integrity Service.
pub mod blobs;
/// Statement creation operations for the Integrity Service.
pub mod statements;

/// Configuration for connecting to the Integrity Service API.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Base URL path for the API (e.g., `https://api.example.com`).
    pub base_path: String,
    /// HTTP client for making requests.
    pub client: reqwest::Client,
    /// Optional bearer token for authentication.
    pub bearer_access_token: Option<String>,
}

/// Basic authentication credentials as (username, optional password).
pub type BasicAuth = (String, Option<String>);

/// API key authentication configuration.
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// Optional prefix for the API key (e.g., "Bearer").
    pub prefix: Option<String>,
    /// The API key value.
    pub key: String,
}

impl Configuration {
    /// Creates a new configuration with default values.
    ///
    /// # Returns
    /// A new `Configuration` with default settings (localhost base path, no auth)
    pub fn new() -> Configuration {
        Configuration::default()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            base_path: "http://localhost".to_owned(),
            client: reqwest::Client::new(),
            bearer_access_token: None,
        }
    }
}
