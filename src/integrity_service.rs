use std::collections::HashMap;

use anyhow::{anyhow, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    cid::{get_multicodec, multicodec},
    lineage::models::statements::{Statement, StatementTrait},
};
use pyo3::prelude::*;
use serde_json;
use uuid::Uuid;

use crate::{
    integrity_service::{
        blobs::{put_blob, put_blob_batch, put_jcs},
        graph::create_graph_record,
        statements::{create_statement, create_statement_batch},
    },
    Context,
};

/// Blob storage operations for the Integrity Service.
pub mod blobs;
/// Context creation operations for the Integrity Service.
pub mod graph;
/// Statement creation operations for the Integrity Service.
pub mod statements;

/// Service for connecting to the Integrity Service API.
#[derive(Debug, Clone)]
#[pyclass]
pub struct Service {
    /// Base URL path for the API (e.g., <https://api.example.com>).
    #[pyo3(get)]
    pub base_path: String,
    /// HTTP client for making requests.
    pub client: reqwest::Client,
    /// Optional bearer token for authentication.
    pub bearer_access_token: Option<String>,
}

/// Basic authentication credentials as (username, optional password).
pub type BasicAuth = (String, Option<String>);

#[pymethods]
impl Service {
    #[staticmethod]
    #[pyo3(signature = (url, api_key=None))]
    /// Creates a service client using the provided URL and API key.
    pub fn new(url: String, api_key: Option<String>) -> Self {
        let token = if let Some(key) = api_key {
            key
        } else {
            std::env::var("EQTY_API_KEY").expect(
                "The api key must be passed to Service.new(), or set in the env var 'EQTY_API_KEY'",
            )
        };

        Service {
            base_path: url,
            client: reqwest::Client::new(),
            bearer_access_token: Some(token),
        }
    }
}

impl Service {
    /// Registers statements with the Integrity Service for the given graph.
    pub async fn register_statements(
        &self,
        graph_id: Uuid,
        statements: Vec<Statement>,
    ) -> Result<()> {
        log::info!(
            "Registering {} statements to {} with graph_id {graph_id:?}",
            statements.len(),
            self.base_path
        );

        if use_batch_register() {
            create_statement_batch(self, graph_id, statements).await?;
            return Ok(());
        }

        for statement in statements {
            let statement_id = statement.get_id();
            log::debug!("Registering statement: {statement_id:?}");

            match create_statement(self, graph_id, statement).await {
                Ok(result) => {
                    log::info!("Registered {statement_id:?} JCS CID {:?}", result.jcs_cid);
                }
                Err(e) => {
                    let msg = format!("Error registering {statement_id:?}: {e:?}");
                    log::error!("{msg}");
                    return Err(anyhow!("msg"));
                }
            };
        }
        Ok(())
    }

    /// Registers a list of graphs with the Integrity Service.
    pub async fn register_graphs(&self, graphs: Vec<Context>) -> Result<()> {
        log::info!("Registering graph tree");
        for graph in graphs {
            log::debug!("Registering graph {}", graph.id);
            create_graph_record(self, graph).await?;
        }
        Ok(())
    }

    /// Registers blobs (by CID) with the Integrity Service.
    pub async fn register_blobs(&self, blob_map: HashMap<String, String>) -> Result<()> {
        if use_batch_register() {
            let mut batch = Vec::with_capacity(blob_map.len());
            for (cid, blob) in blob_map {
                let multicodec = get_multicodec(&cid)?;
                batch.push((cid, blob, multicodec));
            }
            put_blob_batch(self, batch).await?;
            return Ok(());
        }

        for (cid, blob) in blob_map {
            let multicodec = get_multicodec(&cid)?;

            let decoded_blob = BASE64.decode(blob)?;

            if multicodec == multicodec::JSON_JCS {
                let blob = String::from_utf8(decoded_blob.clone())?;
                log::debug!("Registering jsc blob: {blob}. CID {cid}");
                let json = serde_json::from_slice(&decoded_blob)?;
                put_jcs(self, json).await?;
            } else {
                put_blob(self, decoded_blob, multicodec).await?;
            }
        }
        Ok(())
    }
}

/// Creates a new service
///
/// # Returns
/// A new `Service` with default settings (localhost base path, no auth)
impl Default for Service {
    fn default() -> Self {
        Service {
            base_path: "http://localhost:3050".to_owned(),
            client: reqwest::Client::new(),
            bearer_access_token: None,
        }
    }
}

fn use_batch_register() -> bool {
    match std::env::var("EQTY_BATCH_REGISTER") {
        Ok(val) => matches!(
            val.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}
