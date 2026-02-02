use std::{collections::HashMap, sync::Mutex};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use integrity::{
    cid::{iroh::compute_blob_cid, multicodec, prepend_urn_cid},
    lineage::models::statements::{ComputationStatement, Statement, StatementTrait},
    signer::SignerType,
};
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{context::ctx, to_py_err};

/// Result of finalizing a stream computation.
///
/// Contains the computation statement CID and the streamed output data.
#[derive(Debug, Clone, Serialize, Deserialize, IntoPyObject)]
pub struct StreamCIDs {
    compute_id: String,
    stream: Vec<u8>,
}

/// `stream` submodule.
#[pymodule]
pub fn stream(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create, m)?)?;
    m.add_function(wrap_pyfunction!(update, m)?)?;
    m.add_function(wrap_pyfunction!(finalize, m)?)?;

    Ok(())
}

/// creates a new computation stream
#[pyfunction]
fn create(
    py: Python<'_>,
    input_cids: Vec<String>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    timestamp: Option<String>,
) -> PyResult<Bound<'_, PyAny>> {
    log::debug!("Creating new stream");
    let fut = async move {
        let uuid = create_stream_computation(input_cids, operated_by, executed_on, timestamp)
            .await
            .map_err(to_py_err)?;

        log::debug!("New stream created. UUID: {uuid:?}");
        Ok(uuid.to_string())
    };
    future_into_py(py, fut)
}

/// updates an existing computation stream with new data
#[pyfunction]
fn update(py: Python<'_>, id: String, chunk: Vec<u8>) -> PyResult<Bound<'_, PyAny>> {
    let id = Uuid::parse_str(&id).map_err(to_py_err)?;

    log::debug!("Updating stream computation with ID: {id:?}");

    let fut = async move {
        update_stream_computation(id, chunk)
            .await
            .map_err(to_py_err)?;

        Ok(())
    };

    future_into_py(py, fut)
}

/// Finalizes the computation stream and creates the ComputationStatement and (optionally) the VC and VCStatement
/// returns the CID of the computation statement
#[pyfunction]
fn finalize(
    py: Python<'_>,
    id: String,
    static_output_cids: Option<Vec<String>>,
    graph_id: Option<String>,
) -> PyResult<Bound<'_, PyAny>> {
    let graph_id = ctx().resolve_graph_id(graph_id).map_err(to_py_err)?;
    let id = Uuid::parse_str(&id).map_err(to_py_err)?;

    log::debug!("Finalizing stream computation with ID: {id:?}");

    let fut = async move {
        let (compute_id, stream) = finalize_stream(id, static_output_cids, &graph_id)
            .await
            .map_err(to_py_err)?;

        Ok(StreamCIDs { compute_id, stream })
    };

    future_into_py(py, fut)
}

static STREAM_CACHE: Lazy<Mutex<HashMap<Uuid, StreamComputation>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Represents a streaming computation with static inputs and dynamic outputs.
///
/// Stream computations allow for processing data in real-time while maintaining
/// integrity attestation of both static inputs and streaming outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamComputation {
    static_input_cids: Vec<String>,
    streamed_output: Option<Vec<u8>>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    timestamp: Option<String>,
}

/// Creates a new stream computation with static inputs and streaming capabilities.
///
/// # Arguments
/// * `static_input_cids` - List of CIDs for static input data
/// * `operated_by` - Optional identifier for the operator performing the computation
/// * `executed_on` - Optional identifier for the execution environment
/// * `timestamp` - Optional timestamp for the computation
///
/// # Returns
/// * `Result<Uuid>` - UUID identifier for the created stream computation
async fn create_stream_computation(
    static_input_cids: Vec<String>,
    operated_by: Option<String>,
    executed_on: Option<String>,
    timestamp: Option<String>,
) -> Result<Uuid> {
    let id = Uuid::new_v4();

    let stream_computation = StreamComputation {
        static_input_cids,
        streamed_output: None,
        operated_by,
        executed_on,
        timestamp,
    };

    save_stream(id, stream_computation).await?;

    Ok(id)
}

/// Updates an existing stream computation by appending new streamed output data.
///
/// # Arguments
/// * `id` - UUID identifier of the stream computation to update
/// * `new_streamed_output` - New data bytes to append to the stream
///
/// # Returns
/// * `Result<()>` - Success or error if stream not found or update fails
async fn update_stream_computation(id: Uuid, new_streamed_output: Vec<u8>) -> Result<()> {
    let mut stream_computation = load_stream(id).await?;

    stream_computation.streamed_output = match stream_computation.streamed_output {
        Some(mut buffer) => {
            buffer.extend(new_streamed_output);
            Some(buffer)
        }
        None => Some(new_streamed_output),
    };

    save_stream(id, stream_computation).await
}

/// Finalizes a stream computation by creating a ComputationStatement and VC.
///
/// # Arguments
/// * `id` - UUID identifier of the stream computation to finalize
/// * `static_output_cids` - Optional list of additional static output CIDs
///
/// # Returns
/// * `Result<(String, Vec<u8>)>` - Tuple of (ComputationStatement CID, Streamed Bytes)
async fn finalize_stream(
    id: Uuid,
    static_output_cids: Option<Vec<String>>,
    graph_id: &Uuid,
) -> Result<(String, Vec<u8>)> {
    let stream_computation = load_stream(id).await?;

    let stream_cid = get_stream_cid(&stream_computation).await?;

    // Combine stream_cid and static_output_cids into a single vector
    let mut output_cids = vec![stream_cid.clone()];
    if let Some(mut cids) = static_output_cids {
        output_cids.append(&mut cids);
    }

    let compute_cid =
        create_statement_from_stream(&stream_computation, output_cids, graph_id).await?;
    let stream = stream_computation.streamed_output.unwrap_or_default();

    delete_stream(id)?;

    Ok((compute_cid, stream))
}

/// Saves the stream computation to in-memory cache
async fn save_stream(id: Uuid, stream_computation: StreamComputation) -> Result<()> {
    let mut cache = STREAM_CACHE.lock().unwrap();
    cache.insert(id, stream_computation);
    Ok(())
}

async fn get_stream_cid(stream_computation: &StreamComputation) -> Result<String> {
    match &stream_computation.streamed_output {
        Some(streamed_output) => {
            let bytes = Bytes::from(streamed_output.clone());
            let cid = compute_blob_cid(&bytes, multicodec::RAW_BINARY).await?;
            prepend_urn_cid(&cid)
        }
        None => Err(anyhow!("The streamed output was empty")),
    }
}

/// Creates a ComputationStatement from the stream computation and saves it on disk
async fn create_statement_from_stream(
    stream_computation: &StreamComputation,
    output_cids: Vec<String>,
    graph_id: &Uuid,
) -> Result<String> {
    let StreamComputation {
        operated_by,
        executed_on,
        timestamp,
        ..
    } = stream_computation.clone();

    let signer = ctx()
        .active_signer
        .ok_or_else(|| to_py_err("No active signer available"))?;

    // If VComp notary is being used, we fetch `operatedBy` and `executedOn`` from the signer
    let (operated_by, executed_on) = match &signer {
        SignerType::VCompNotarySigner(signer) => {
            let operated_by = if operated_by.is_some() {
                operated_by
            } else {
                signer.operated_by.clone()
            };
            let executed_on = if executed_on.is_some() {
                executed_on
            } else {
                signer.executed_on.clone()
            };

            (operated_by, executed_on)
        }
        _ => (operated_by, executed_on),
    };

    let registered_by = signer.get_did_doc().id;
    let operated_by = match operated_by {
        Some(operated_by) => operated_by,
        None => registered_by.clone(),
    };

    let statement = Statement::ComputationRegistration(
        ComputationStatement::create(
            None,
            stream_computation.static_input_cids.clone(),
            output_cids,
            operated_by,
            executed_on,
            registered_by,
            timestamp,
        )
        .await?,
    );

    ctx()
        .sql_lite
        .register_statement(&statement, graph_id)
        .await?;

    Ok(statement.get_id())
}

/// Deletes the stream computation from in-memory cache
fn delete_stream(id: Uuid) -> Result<()> {
    let mut cache = STREAM_CACHE.lock().unwrap();
    cache.remove(&id);
    Ok(())
}

/// Loads the stream computation from in-memory cache
async fn load_stream(id: Uuid) -> Result<StreamComputation> {
    let cache = STREAM_CACHE.lock().unwrap();
    cache
        .get(&id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Stream computation not found for id={}", id))
}
