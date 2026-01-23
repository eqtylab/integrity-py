use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::{self, BlobStore},
    cid::{get_multicodec, multicodec},
    lineage::{
        graph_indexer::sql_indexer::IStatementIdx,
        models::{
            graph::Graph,
            manifest::{
                generate_manifest as lineage_generate_manifest,
                manifest_v4::{generate_manifest_v4, ManifestV4},
                merge_async, Manifest,
            },
            statements::Statement,
        },
    },
};
use pyo3::{
    prelude::*,
    pyfunction, pymodule,
    types::{PyBytes, PyDict, PyList, PyModule},
    wrap_pyfunction, PyResult, Python,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    context::{self, ctx},
    convert_attributes,
    feature_flags::FeatureFlags,
    integrity_service::{
        blobs::put_jcs,
        statements::{create_statement, CreateStatementRequestBody},
        Configuration,
    },
    to_py_err,
};

/// `manifest` submodule.
#[pymodule]
pub fn manifest(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(generate_v4, m)?)?;
    m.add_function(wrap_pyfunction!(import_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(merge, m)?)?;
    m.add_function(wrap_pyfunction!(register, m)?)?;

    Ok(())
}

/// Generates an integrity graph manifest json string
#[pyfunction]
pub fn generate(
    py: Python,
    statements: Vec<PyObject>,
    blobs_dir: PathBuf,
    attributes: Option<PyObject>,
    include_context: Option<bool>,
) -> PyResult<String> {
    if FeatureFlags::is_enabled("graph_ids") {
        let msg = "Feature 'graph_ids' must be disabled to use this fn.".to_string();
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg));
    }
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

    // Convert PyObject attributes to HashMap
    let attributes_map = if let Some(attr) = attributes {
        let json_value = python_to_json_value(py, &attr)?;
        match json_value {
            Value::Object(map) => Some(map.into_iter().collect::<HashMap<String, Value>>()),
            Value::Null => None,
            _ => {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "attributes must be a dict or None",
                ))
            }
        }
    } else {
        None
    };

    let manifest = context::get_runtime()
        .block_on(lineage_generate_manifest(
            include_context.unwrap_or(true),
            rust_statements,
            attributes_map,
            blobs,
        ))
        .map_err(to_py_err)?;

    let manifest_json = serde_json::to_string(&manifest).map_err(to_py_err)?;
    Ok(manifest_json)
}

/// Generates a v4 integrity graph manifest JSON string from multiple graphs.
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
pub fn generate_v4(
    py: Python,
    graphs: &PyList,
    blobs_dir: PathBuf,
    include_context: Option<bool>,
) -> PyResult<String> {
    if !FeatureFlags::is_enabled("graph_ids") {
        let msg = "Feature 'graph_ids' must be enabled to use this fn.".to_string();
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg));
    }

    let include_context = include_context.unwrap_or(false);

    // Convert Python graphs list to Rust Graph structs
    let mut rust_graphs: Vec<Graph> = Vec::new();
    for graph_item in graphs {
        let graph_dict = graph_item.downcast::<PyDict>().map_err(to_py_err)?;

        // Extract graph metadata
        let id_str: String = graph_dict
            .get_item("id")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'id' field"))?
            .extract()?;
        let id = Uuid::parse_str(&id_str).map_err(to_py_err)?;

        let name: String = graph_dict
            .get_item("name")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'name' field"))?
            .extract()?;

        let parent: Option<String> = graph_dict.get_item("parent").and_then(|p| p.extract().ok());
        let parent = parent
            .map(|p| Uuid::parse_str(&p))
            .transpose()
            .map_err(to_py_err)?;

        // Extract and convert statements
        let statements_list = graph_dict
            .get_item("statements")
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'statements' field")
            })?
            .downcast::<PyList>()
            .map_err(to_py_err)?;

        let mut statements: Vec<Statement> = Vec::new();
        for stmt_item in statements_list {
            // Convert Python statement dict back to JSON, then to Statement
            let stmt_json = python_to_json_value(py, &stmt_item.into())?;
            let statement: Statement = serde_json::from_value(stmt_json).map_err(to_py_err)?;
            statements.push(statement);
        }

        rust_graphs.push(Graph {
            id,
            name,
            parent,
            statements: Some(statements),
        });
    }

    // Get all statements from all graphs for blob resolution
    let all_statements: Vec<Statement> = rust_graphs
        .iter()
        .filter_map(|graph| graph.statements.as_ref())
        .flatten()
        .cloned()
        .collect();

    // Read blobs from directory
    let blobs = context::get_runtime()
        .block_on(resolve_blobs(&all_statements, blobs_dir))
        .map_err(to_py_err)?;

    log::info!("Generating manifest v4 for {} graphs", rust_graphs.len());

    // Generate the manifest
    let manifest = context::get_runtime()
        .block_on(generate_manifest_v4(include_context, rust_graphs, blobs))
        .map_err(to_py_err)?;

    // Convert manifest to JSON string
    let manifest_json = serde_json::to_string(&manifest).map_err(to_py_err)?;

    Ok(manifest_json)
}

/// Imports a manifest and eturns the decoded blobs that must be saved
#[pyfunction]
pub fn import_manifest<'py>(
    py: Python<'py>,
    manifest: String,
    attributes: &'py PyDict,
) -> PyResult<HashMap<String, &'py PyBytes>> {
    let attributes = convert_attributes(attributes).map_err(to_py_err)?;
    log::info!("Importing manifest with attributes {:?}", attributes);

    let blobs = context::get_runtime()
        .block_on(rust_import(manifest, Some(&attributes)))
        .map_err(to_py_err)?;

    let py_blobs = blobs
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

fn python_to_json_value(py: Python, obj: &PyObject) -> PyResult<Value> {
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
    } else if let Ok(py_list) = obj.downcast::<PyList>(py) {
        let mut vec = Vec::new();
        for item in py_list {
            vec.push(python_to_json_value(py, &item.into())?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(py_dict) = obj.downcast::<PyDict>(py) {
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

async fn rust_import(
    manifest: String,
    attributes: Option<&HashMap<String, Value>>,
) -> Result<HashMap<String, Vec<u8>>> {
    if FeatureFlags::is_enabled("graph_ids") {
        let manifest = serde_json::from_str::<ManifestV4>(&manifest)?;
        log::trace!("Importing manifest V4");

        for graph in manifest.graphs {
            ctx()
                .sql_lite2
                .create_graph(&graph.id, &graph.name, graph.parent.as_ref())
                .await?;

            if let Some(statements) = graph.statements {
                for statement in statements {
                    ctx()
                        .register_statement_locally(statement.clone(), None, Some(&graph.id))
                        .await?
                }
            }
        }

        log::debug!("Decoding {} blobs", manifest.blobs.len());
        // Decode base64 values in the blobs HashMap
        let decoded_blobs = manifest
            .blobs
            .into_iter()
            .map(|(key, base64_value)| {
                log::debug!("Decoding blob key: {}", key);
                log::trace!("Base64 value length: {} chars", base64_value.len());
                let decoded_bytes = BASE64.decode(&base64_value)?;
                log::debug!("Decoded {} bytes for key: {key}", decoded_bytes.len());

                Ok((key, decoded_bytes))
            })
            .collect::<Result<HashMap<String, Vec<u8>>, anyhow::Error>>()?;

        Ok(decoded_blobs)
    } else {
        let manifest = serde_json::from_str::<Manifest>(&manifest)?;
        log::trace!("Manifest str: \n{manifest:?}");
        log::debug!("{} statements imported", manifest.statements.keys().len());
        for statement in manifest.statements.values() {
            ctx()
                .register_statement_locally(statement.clone(), attributes, None)
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
        if let Some(ref attributes) = manifest.attributes {
            let a = attributes.get(&statement.0);
            register_statement(config, statement.1, a).await?;
        } else {
            register_statement(config, statement.1, None).await?;
        }
    }

    for blob in manifest.blobs.into_iter() {
        register_blob(config, blob.0, blob.1).await?;
    }
    Ok(())
}

async fn register_statement(
    ig_service_config: &Configuration,
    statement: Statement,
    attributes: Option<&Value>,
) -> Result<()> {
    log::debug!("Registering statement: {statement:?}");

    let statement_str = serde_json::to_value(&statement)?;
    let body = CreateStatementRequestBody::new(attributes.cloned(), Some(statement_str));

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
