use anyhow::{anyhow, Result};
use integrity::lineage::models::statements::Statement;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::Service;

/// Request body for creating a statement via the Integrity Service API.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateStatementRequestBody {
    /// Whether to auto-generate an ID for the statement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_id: Option<Option<bool>>,
    /// Whether to issue a verifiable credential for the statement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_vc: Option<Option<bool>>,
    /// The statement payload as JSON.
    #[serde(deserialize_with = "Option::deserialize")]
    pub statement: Option<Value>,
    /// Context identifier for statement association.
    pub graph_id: Uuid,
}

impl CreateStatementRequestBody {
    /// Creates a new request body for the create statement API.
    ///
    /// # Arguments
    /// * `statement` - Optional JSON representation of the statement
    ///
    /// # Returns
    /// A new `CreateStatementRequestBody` with the provided values
    pub fn new(graph_id: Uuid, statement: Option<Value>) -> CreateStatementRequestBody {
        CreateStatementRequestBody {
            generate_id: None,
            issue_vc: None,
            statement,
            graph_id,
        }
    }
}

/// Response from the create statement API containing the computed CIDs.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateStatementResponse {
    /// The JCS (JSON Canonicalization Scheme) content identifier.
    pub jcs_cid: String,
    /// The RDFC (RDF Dataset Canonicalization) content identifier.
    pub rdfc_cid: String,
}

impl CreateStatementResponse {
    /// Creates a new response with the provided CIDs.
    ///
    /// # Arguments
    /// * `jcs_cid` - The JCS content identifier
    /// * `rdfc_cid` - The RDFC content identifier
    ///
    /// # Returns
    /// A new `CreateStatementResponse` with the provided CIDs
    pub fn new(jcs_cid: String, rdfc_cid: String) -> CreateStatementResponse {
        CreateStatementResponse { jcs_cid, rdfc_cid }
    }
}

/// Creates a statement via the Integrity Service API.
///
/// # Arguments
/// * `service` - API configuration containing base path and authentication
/// * `statements_create_request_body` - Request body containing the statement and attributes
///
/// # Returns
/// * `Result<CreateStatementResponse>` - The computed CIDs on success, or an error on failure
pub async fn create_statement(
    service: &Service,
    statement: Statement,
    graph_id: Uuid,
    // statements_create_request_body: CreateStatementRequestBody,
) -> Result<CreateStatementResponse> {
    let statement_str = serde_json::to_value(&statement)?;
    let body = CreateStatementRequestBody::new(graph_id, Some(statement_str));
    let uri_str = format!("{}/statements/v1", service.base_path);
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
        serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse create statement response: {e}"))
    } else {
        Err(anyhow!(
            "Create Statement request failed with status {status}: {content}"
        ))
    }
}
