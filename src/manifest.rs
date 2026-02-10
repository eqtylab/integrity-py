use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context as AnyhowContext, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::{self, BlobStore},
    cid::{get_multicodec, multicodec},
    lineage::models::{
        manifest::{generate_manifest, merge_async, Manifest as IntegrityManifest},
        statements::Statement,
    },
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyType},
    Bound,
};
use pyo3_async_runtimes::tokio::get_runtime;
use serde_json::Value;

use crate::{
    integrity_service::{
        blobs::put_jcs,
        statements::{create_statement, CreateStatementRequestBody},
        Configuration,
    },
    with_ctx,
};

/// `manifest` submodule.
#[pymodule]
pub fn manifest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Manifest>()?;
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(merge, m)?)?;
    m.add_function(wrap_pyfunction!(register, m)?)?;

    Ok(())
}

#[pyclass(name = "Manifest")]
pub struct Manifest {
    #[pyo3(get)]
    manifest_str: String,
}

#[pymethods]
impl Manifest {
    #[new]
    fn new(manifest: String) -> Self {
        Self {
            manifest_str: manifest,
        }
    }

    #[classmethod]
    #[pyo3(signature = (statements, include_context=true))]
    fn from_statements(
        _cls: &Bound<'_, PyType>,
        py: Python,
        statements: Py<PyAny>,
        include_context: bool,
    ) -> PyResult<Self> {
        let graphs_any = statements.getattr(py, "graphs")?;
        let graphs: Vec<Py<PyAny>> = graphs_any.extract(py)?;
        let blobs_dir = crate::config::get_blob_dir()?;
        let manifest_str = generate(py, graphs, blobs_dir, Some(include_context))?;
        Ok(Self { manifest_str })
    }

    fn export(&self, file: PathBuf) -> PyResult<()> {
        std::fs::write(&file, &self.manifest_str).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to write manifest: {e}"))
        })
    }

    #[classmethod]
    #[pyo3(signature = (manifest))]
    fn import_manifest(_cls: &Bound<'_, PyType>, py: Python, manifest: Py<PyAny>) -> PyResult<()> {
        let manifest_str = if let Ok(s) = manifest.extract::<String>(py) {
            s
        } else if let Ok(path) = manifest.extract::<PathBuf>(py) {
            std::fs::read_to_string(&path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to read manifest file {}: {e}",
                    path.display()
                ))
            })?
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Manifest must be a string or Path",
            ));
        };

        let blobs: HashMap<String, Vec<u8>> = with_ctx!(py, |ctx| {
            let graph_id = ctx.resolve_graph_id(None);
            let manifest = serde_json::from_str::<IntegrityManifest>(&manifest_str)?;
            log::trace!("Manifest str: \n{manifest:?}");
            log::debug!("{} statements imported", manifest.statements.keys().len());
            for statement in manifest.statements.values() {
                ctx.sql_lite
                    .register_statement(statement, &graph_id)
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
            Ok::<_, anyhow::Error>(decoded_blobs)
        })?;

        let blob_dir = crate::config::get_blob_dir()?;
        for (blob_key, blob_content) in blobs {
            let blob_file_path = blob_dir.join(blob_key);
            std::fs::write(&blob_file_path, blob_content).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to write blob {}: {e}",
                    blob_file_path.display()
                ))
            })?;
        }

        Ok(())
    }

    #[staticmethod]
    fn merge(py: Python, a: String, b: String) -> PyResult<String> {
        merge(py, a, b)
    }

    fn register(&self, py: Python) -> PyResult<()> {
        let api_key = env::var("EQTY_API_KEY")
            .map_err(|_| usage_error(py, "The env var 'EQTY_API_KEY' must be set"))?;
        register(py, self.manifest_str.clone(), Some(api_key))
    }
}

fn usage_error(py: Python, msg: &str) -> PyErr {
    let usage_error = (|| -> PyResult<PyErr> {
        let module = py.import("eqty_sdk.errors")?;
        let exc = module.getattr("UsageError")?;
        let instance = exc.call1((msg,))?;
        Ok(PyErr::from_value(instance))
    })();

    usage_error.unwrap_or_else(|_| pyo3::exceptions::PyRuntimeError::new_err(msg.to_string()))
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
            serde_json::from_value(json_value)
                .context("Failed to deserialize statement")
                .map_err(Into::into)
        })
        .collect();

    let rust_statements = rust_statements?;

    py.detach(|| {
        get_runtime().block_on(async {
            let blobs = resolve_blobs(&rust_statements, blobs_dir).await?;

            log::info!("Generating manifest");

            let manifest =
                generate_manifest(include_context.unwrap_or(true), rust_statements, blobs).await?;

            let manifest_json =
                serde_json::to_string(&manifest).context("Failed to serialize manifest")?;
            Ok(manifest_json)
        })
    })
}

/// Merges the manifests `a` and `b` and returns the merged manifest.
#[pyfunction]
pub fn merge(py: Python, a: String, b: String) -> PyResult<String> {
    py.detach(|| {
        get_runtime().block_on(async {
            let a = serde_json::from_str(&a).context("Failed to parse first manifest")?;
            let b = serde_json::from_str(&b).context("Failed to parse second manifest")?;
            let manifest = merge_async(a, b).await?;
            let manifest_str =
                serde_json::to_string(&manifest).context("Failed to serialize merged manifest")?;
            Ok(manifest_str)
        })
    })
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
    } else if let Ok(py_list) = obj.cast_bound::<PyList>(py) {
        let mut vec = Vec::new();
        for item in py_list {
            vec.push(python_to_json_value(py, &item.into())?);
        }
        Ok(Value::Array(vec))
    } else if let Ok(py_dict) = obj.cast_bound::<PyDict>(py) {
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

/// Register the manfiest with integrity platform
#[pyfunction]
fn register(py: Python, manifest: String, api_key: Option<String>) -> PyResult<()> {
    with_ctx!(py, |ctx| {
        let manifest = serde_json::from_str::<IntegrityManifest>(&manifest)
            .context("Failed to parse manifest")?;
        let ig_service_config = ctx.get_integrity_service_config(api_key)?;

        register_async(&ig_service_config, manifest).await?;
        Ok(())
    })
}

async fn register_async(config: &Configuration, manifest: IntegrityManifest) -> Result<()> {
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
