use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context as AnyhowContext, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::BlobStore,
    cid::get_multicodec,
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

use crate::with_ctx;

/// `manifest` submodule.
#[pymodule]
pub fn manifest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Manifest>()?;
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(merge, m)?)?;

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

        let manifest_str = generate(py, graphs, Some(include_context))?;
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

        with_ctx!(py, |ctx| {
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

            for (blob_key, blob_content) in decoded_blobs {
                let codec = get_multicodec(&blob_key)?;
                ctx.blob_store
                    .put(blob_content, codec, Some(&blob_key))
                    .await?;
            }

            Ok::<_, anyhow::Error>(())
        })?;

        Ok(())
    }

    #[staticmethod]
    fn merge(py: Python, a: String, b: String) -> PyResult<String> {
        merge(py, a, b)
    }
}

/// Generates an integrity graph manifest JSON string from multiple graphs.
///
/// # Arguments
/// * `py` - Python interpreter reference
/// * `graphs` - Python list of graph dictionaries, each containing 'id', 'name', 'parent', and 'statements'
/// * `include_context` - Whether to include context information in the manifest (default: false)
///
/// # Returns
/// * `PyResult<String>` - JSON string representation of the manifest, or error on failure
#[pyfunction]
pub fn generate(
    py: Python,
    statements: Vec<Py<PyAny>>,
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

    let manifest_json = with_ctx!(py, |ctx| {
        let blob_store = Arc::new(ctx.blob_store.clone());
        let blobs =
            integrity::lineage::models::manifest::resolve_blobs(&rust_statements, blob_store, 8)
                .await?;

        log::info!("Generating manifest");

        let manifest =
            generate_manifest(include_context.unwrap_or(true), rust_statements, blobs).await?;

        let manifest_json =
            serde_json::to_string(&manifest).context("Failed to serialize manifest")?;
        Ok::<_, anyhow::Error>(manifest_json)
    })?;

    Ok(manifest_json)
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
