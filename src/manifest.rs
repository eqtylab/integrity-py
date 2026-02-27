use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::BlobStore,
    cid::get_multicodec,
    lineage::models::manifest::{merge_async, Manifest as IntegrityManifest},
};
use pyo3::{prelude::*, types::PyType, Bound};
use pyo3_async_runtimes::tokio::get_runtime;

use crate::with_ctx;

/// `manifest` submodule.
#[pymodule]
pub fn manifest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Manifest>()?;
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
