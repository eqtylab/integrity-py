use anyhow::{anyhow, Result};
use serde::{self, Deserialize, Serialize};
use serde_json::{self, Value};

use super::Service;

/// Response from the blob store PUT APIs containing the computed CID.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PutResponse {
    /// The content identifier for the stored blob.
    pub cid: String,
}

impl PutResponse {
    /// Creates a new response with the provided CID.
    ///
    /// # Arguments
    /// * `cid` - The content identifier for the stored blob
    ///
    /// # Returns
    /// A new `PutResponse` with the provided CID
    pub fn new(cid: String) -> PutResponse {
        PutResponse { cid }
    }
}

/// Stores a JCS-serialized JSON document in the blob store.
///
/// The provided JSON will be serialized using JSON Canonicalization Scheme (JCS)
/// and then stored in the blob store.
///
/// # Arguments
/// * `service` - API configuration containing base path and authentication
/// * `body` - JSON value to serialize and store
///
/// # Returns
/// * `Result<PutResponse>` - The CID of the stored blob on success, or an error on failure
pub async fn put_jcs(service: &Service, body: Value) -> Result<PutResponse> {
    let uri_str = format!("{}/store/v1/jcs", service.base_path);
    let mut req_builder = service.client.request(reqwest::Method::PUT, &uri_str);

    if let Some(ref token) = service.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder.json(&body);

    let req = req_builder.build()?;
    let resp = service.client.execute(req).await?;

    let status = resp.status();

    let content = resp.text().await?;
    if status.is_success() {
        serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse put jcs response: {e}"))
    } else {
        Err(anyhow!(
            "Put JCS request failed with status {status}: {content}"
        ))
    }
}

/// Stores a raw blob in the blob store with the specified multicodec.
///
/// # Arguments
/// * `service` - API configuration containing base path and authentication
/// * `blob` - Raw byte data to store
/// * `multicodec` - Multicodec identifier indicating the blob's format
///
/// # Returns
/// * `Result<PutResponse>` - The CID of the stored blob on success, or an error on failure
pub async fn put_blob(service: &Service, blob: Vec<u8>, multicodec: u64) -> Result<PutResponse> {
    let uri_str = format!("{}/store/v1/blob", service.base_path);
    let mut req_builder = service.client.request(reqwest::Method::PUT, &uri_str);

    if let Some(ref token) = service.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder
        .body(blob)
        .query(&[("multicodec_code", multicodec)]);

    let req = req_builder.build()?;
    let resp = service.client.execute(req).await?;

    let status = resp.status();

    let content = resp.text().await?;
    if status.is_success() {
        serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse put blob response: {e}"))
    } else {
        Err(anyhow!(
            "Put Blob request failed with status {status}: {content}"
        ))
    }
}
