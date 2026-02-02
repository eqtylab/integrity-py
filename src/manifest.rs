use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::{self, BlobStore},
    cid::{get_multicodec, multicodec},
    lineage::models::{
        manifest::{generate_manifest, merge_async, Manifest},
        statements::Statement,
    },
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::Bound;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    context::{self, ctx},
    integrity_service::{
        blobs::put_jcs,
        statements::{create_statement, CreateStatementRequestBody},
        Configuration,
    },
    to_py_err,
};

/// `manifest` submodule.
#[pymodule]
pub fn manifest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(import_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(merge, m)?)?;
    m.add_function(wrap_pyfunction!(register, m)?)?;

    Ok(())
}

/// Generates an integrity graph manifest JSON string from multiple graphs.
///
/// # Arguments
/// * `py` - Python interpreter reference
/// * `graphs` - Python list of graph dictionaries, each containing 'id', 'name', 'parent', and 'statements'
/// * `blobs_dir` - Path to directory containing blob files referenced by statements
/// * `include_context` - Whether to include context information in the manifest (default: false)
///
/// # Returns
/// * `PyResult<String>` - JSON string representation of the manifest, or error on failure
#[pyfunction]
pub fn generate(
    py: Python,
    statements: Vec<Py<PyAny>>,
    blobs_dir: PathBuf,
    include_context: Option<bool>,
) -> PyResult<String> {
    // Convert PyObjects to Statements
    let rust_statements: PyResult<Vec<Statement>> = statements
        .into_iter()
        .map(|py_obj| {
            let json_value = python_to_json_value(py, &py_obj)?;
            serde_json::from_value(json_value).map_err(to_py_err)
        })
        .collect();

    let rust_statements = rust_statements?;

    let blobs = context::get_runtime()
        .block_on(resolve_blobs(&rust_statements, blobs_dir))
        .map_err(to_py_err)?;

    log::info!("Generating manifest");

    let manifest = context::get_runtime()
        .block_on(generate_manifest(
            include_context.unwrap_or(true),
            rust_statements,
            blobs,
        ))
        .map_err(to_py_err)?;

    let manifest_json = serde_json::to_string(&manifest).map_err(to_py_err)?;
    Ok(manifest_json)
}

/// Imports a manifest and returns the decoded blobs that must be saved
#[pyfunction]
pub fn import_manifest<'py>(
    py: Python<'py>,
    manifest: String,
    graph_id: Option<String>,
) -> PyResult<HashMap<String, Bound<'py, PyBytes>>> {
    let graph_id = ctx().resolve_graph_id(graph_id).map_err(to_py_err)?;
    let blobs = context::get_runtime()
        .block_on(rust_import(manifest, &graph_id))
        .map_err(to_py_err)?;

    // Convert Vec<u8> to PyBytes
    let py_blobs: HashMap<String, Bound<'py, PyBytes>> = blobs
        .into_iter()
        .map(|(k, v)| (k, PyBytes::new(py, &v)))
        .collect();

    Ok(py_blobs)
}

/// Merges the manifests `a` and `b` and returns the merged manifest.
#[pyfunction]
pub fn merge(_py: Python, a: String, b: String) -> PyResult<String> {
    fn rust_merge(a: String, b: String) -> Result<String> {
        let a = serde_json::from_str(&a)?;
        let b = serde_json::from_str(&b)?;
        let manifest = context::get_runtime().block_on(merge_async(a, b))?;
        let manifest_str = serde_json::to_string(&manifest)?;
        Ok(manifest_str)
    }

    rust_merge(a, b).map_err(to_py_err)
}

fn python_to_json_value(py: Python, obj: &Py<PyAny>) -> PyResult<Value> {
    if obj.is_none(py) {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>(py) {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>(py) {
        Ok(Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>(py) {
        Ok(Value::Number(
            serde_json::Number::from_f64(f).unwrap_or(0.into()),
        ))
    } else if let Ok(s) = obj.extract::<String>(py) {
        Ok(Value::String(s))
    } else if let Ok(py_list) = obj.bind(py).downcast::<PyList>() {
        let mut vec = Vec::new();
        for item in py_list {
            vec.push(python_to_json_value(py, &item.into())?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(py_dict) = obj.bind(py).downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in py_dict {
            let key_str = key.extract::<String>()?;
            map.insert(key_str, python_to_json_value(py, &value.into())?);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported Python type for JSON conversion",
        ))
    }
}

async fn rust_import(manifest: String, graph_id: &Uuid) -> Result<HashMap<String, Vec<u8>>> {
    let manifest = serde_json::from_str::<Manifest>(&manifest)?;
    log::trace!("Manifest str: \n{manifest:?}");
    log::debug!("{} statements imported", manifest.statements.keys().len());
    for statement in manifest.statements.values() {
        ctx()
            .sql_lite
            .register_statement(statement, graph_id)
            .await?
    }
    // Decode base64 values in the blobs HashMap
    let decoded_blobs = manifest
        .blobs
        .into_iter()
        .map(|(key, base64_value)| {
            let decoded_bytes = BASE64.decode(&base64_value)?;
            Ok((key, decoded_bytes))
        })
        .collect::<Result<HashMap<String, Vec<u8>>, anyhow::Error>>()?;

    Ok(decoded_blobs)
}

/// Register the manfiest with integrity platform
#[pyfunction]
fn register(_py: Python, manifest: String, api_key: Option<String>) -> PyResult<()> {
    let manifest = serde_json::from_str::<Manifest>(&manifest).map_err(to_py_err)?;
    let ig_service_config = ctx()
        .get_integrity_service_config(api_key)
        .map_err(to_py_err)?;

    context::get_runtime()
        .block_on(register_async(&ig_service_config, manifest))
        .map_err(to_py_err)?;
    Ok(())
}

async fn register_async(config: &Configuration, manifest: Manifest) -> Result<()> {
    for statement in manifest.statements.into_iter() {
        register_statement(config, statement.1).await?;
    }

    for blob in manifest.blobs.into_iter() {
        register_blob(config, blob.0, blob.1).await?;
    }
    Ok(())
}

async fn register_statement(ig_service_config: &Configuration, statement: Statement) -> Result<()> {
    log::debug!("Registering statement: {statement:?}");

    let statement_str = serde_json::to_value(&statement)?;
    let body = CreateStatementRequestBody::new(Some(statement_str));

    match create_statement(ig_service_config, body).await {
        Ok(result) => {
            log::info!("Registered {statement:?} JCS CID {:?}", result.jcs_cid);
            Ok(())
        }
        Err(e) => {
            let msg = format!("Error registering {statement:?}: {e:?}");
            log::error!("{msg}");
            Err(anyhow!("msg"))
        }
    }
}

async fn register_blob(ig_service_config: &Configuration, cid: String, blob: String) -> Result<()> {
    let multicodec = get_multicodec(&cid)?;

    let decoded_blob = BASE64.decode(blob)?;

    if multicodec == multicodec::JSON_JCS {
        let blob = String::from_utf8(decoded_blob.clone())?;
        log::debug!("Registering jsc blob: {blob}. CID {cid}");
        let json = serde_json::from_slice(&decoded_blob)?;
        put_jcs(ig_service_config, json).await?;
    } else {
        reqwest::Client::new()
            .put(format!("{}/store/v1/blob", ig_service_config.base_path))
            // TODO: clean this up, would be better to fix the opeanpi generated code and use that
            //       instead of handling auth differently here
            .headers(
                if let Some(api_key) = &ig_service_config.bearer_access_token {
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}"))?,
                    );
                    headers
                } else {
                    reqwest::header::HeaderMap::new()
                },
            )
            .body(decoded_blob)
            .query(&[("multicodec_code", multicodec)])
            .send()
            .await?;
    }
    Ok(())
}

/// Gets the blobs that are referenced by the statements
async fn resolve_blobs(
    statements: &Vec<Statement>,
    blobs_dir: PathBuf,
) -> Result<HashMap<String, String>> {
    let mut blob_store = blob_store::LocalFs::new(blobs_dir);
    blob_store.init().await?;

    let blob_store = Arc::new(blob_store);
    integrity::lineage::models::manifest::resolve_blobs(statements, blob_store, 8).await
}
