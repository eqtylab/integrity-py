use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::Service;
use crate::indexer::Context;

/// Request body for registering a graph with the Integrity Service.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphRegistrationRequest {
    #[serde(rename = "graph_id")]
    /// Graph UUID as a string.
    pub graph_id: String,
    #[serde(rename = "name")]
    /// Human-readable graph name.
    pub name: String,
    #[serde(rename = "parent_id", default, skip_serializing_if = "Option::is_none")]
    /// Optional parent graph UUID.
    pub parent_id: Option<Option<String>>,
}

impl GraphRegistrationRequest {
    /// Creates a new graph registration request.
    pub fn new(graph_id: String, name: String) -> GraphRegistrationRequest {
        GraphRegistrationRequest {
            graph_id,
            name,
            parent_id: None,
        }
    }
}

/// Creates or updates a graph record via the Integrity Service API.
pub async fn create_graph_record(service: &Service, context: Context) -> Result<()> {
    let mut body = GraphRegistrationRequest::new(context.id.to_string(), context.name);
    if context.parent.is_some() {
        body.parent_id = Some(context.parent.map(|id| id.to_string()));
    }
    log::info!("Creating graph record. {body:?}");

    let uri_str = format!("{}/graph/v1", service.base_path);
    let mut req_builder = service.client.request(reqwest::Method::PUT, &uri_str);

    if let Some(ref token) = service.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder.json(&body);

    let req = req_builder.build()?;
    let resp = service.client.execute(req).await?;

    let status = resp.status();

    match status {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {
            log::info!("Graph {} created successfully", context.id);
            Ok(())
        }
        reqwest::StatusCode::CONFLICT => {
            log::warn!("Graph {} is already registered", context.id);
            Ok(())
        }
        reqwest::StatusCode::BAD_REQUEST => {
            let status = resp.status();
            let content = resp.text().await?;
            Err(anyhow!(
                "Failed to register graph {}: HTTP {status} - {content}",
                context.id
            ))
        }
        _ => Err(anyhow!("Failed to register graph {}: {resp:?}", context.id)),
    }
}
