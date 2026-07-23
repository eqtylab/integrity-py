//! EQTY SDK - Python bindings for the Integrity framework.
//!
//! This crate provides Python bindings via PyO3 for cryptographic signing,
//! content-addressed storage, and lineage statement management functionality.

use std::{env, path::PathBuf};

use config::Config;
use integrity::{
    blob_store::BlobStore,
    cid::{
        blake3::blake3_cid_raw_binary,
        get_multicodec,
        iroh::{compute_dir_cid, compute_file_cid},
        jcs::compute_jcs_cid,
        multicodec,
    },
};
use pyo3::exceptions::PyRuntimeError;
use pyo3_async_runtimes::tokio::get_runtime;
use serde_json::Value;
use tokio::fs;

use crate::cid::CID;

/// Resolves skip_proof from provided option or EQTY_SKIP_PROOF environment variable.
///
/// Returns true if:
/// - `skip_proof` is Some(true), or
/// - `skip_proof` is None and EQTY_SKIP_PROOF env var is "true" (case-insensitive)
pub fn resolve_skip_proof(skip_proof: Option<bool>) -> bool {
    skip_proof.unwrap_or_else(|| {
        env::var("EQTY_SKIP_PROOF")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

/// Resolves timestamp from provided option or EQTY_TIMESTAMP environment variable.
///
/// Returns:
/// - The provided timestamp if Some
/// - The EQTY_TIMESTAMP env var value if set
/// - None otherwise
pub fn resolve_timestamp(timestamp: Option<String>) -> Option<String> {
    timestamp.or_else(|| env::var("EQTY_TIMESTAMP").ok())
}

/// Content identifier (CID) computation and utilities.
pub mod cid;
/// Global application configuration management.
pub mod config;
/// DID type for registering DID statements and metadata.
pub mod did;
/// Entity type for unhashed objects with UUID identifiers.
pub mod entity;
/// Indexes integrity information in sql database.
pub mod indexer;
/// API functions for connecting to the integrity service.
pub mod integrity_service;
/// Cryptographic signer creation and management.
pub mod signer;
/// Lineage statement creation and storage operations.
pub mod statements;
/// Streaming computation support for real-time data processing.
pub mod stream;
/// UUID type for urn:uuid identifiers.
pub mod uuid;
/// Offline verification for credentials and lineage statement identifiers.
pub mod verification;

use pyo3::{prelude::*, wrap_pymodule};

use crate::{indexer::Context, uuid::UUID};

fn suppress_noisy_integrity_loggers(py: Python<'_>) -> PyResult<()> {
    let logging = py.import("logging")?;
    let get_logger = logging.getattr("getLogger")?;
    let error_level = logging.getattr("ERROR")?;

    // Missing blobs are common during manifest export, so suppress that logger by default.
    let manifest_logger = get_logger.call1(("integrity_lineage_models.models.manifest",))?;
    manifest_logger.call_method1("setLevel", (error_level,))?;

    Ok(())
}

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize pyo3-log to route Rust log messages to Python's logging module
    let _ = pyo3_log::try_init();
    suppress_noisy_integrity_loggers(m.py())?;

    m.add_wrapped(wrap_pymodule!(entity::entity))?;
    m.add_wrapped(wrap_pymodule!(signer::signer))?;
    m.add_wrapped(wrap_pymodule!(statements::statements))?;
    m.add_wrapped(wrap_pymodule!(stream::stream))?;

    m.add_class::<cid::CID>()?;
    m.add_class::<UUID>()?;
    m.add_class::<Context>()?;
    m.add_class::<indexer::ContextFactory>()?;
    m.add_class::<Config>()?;
    m.add_class::<did::DID>()?;
    m.add_class::<signer::Signer>()?;
    m.add_class::<signer::SignerAlgorithms>()?;
    m.add_class::<entity::Entity>()?;
    m.add_class::<integrity_service::Service>()?;
    m.add_class::<statements::PyAssociationType>()?;

    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_json, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_path, m)?)?;
    m.add_function(wrap_pyfunction!(verification::verify_vc, m)?)?;
    m.add_function(wrap_pyfunction!(
        verification::verify_statement_rdfc_cid,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(purge_statement_store, m)?)?;
    m.add_function(wrap_pyfunction!(purge_blob_store, m)?)?;
    Ok(())
}

/// Initializes the sdk config. Must be called before setting individual config values
#[pyfunction]
#[pyo3(signature = (default_context=None, *, custom_dir=None))]
fn init(
    py: Python<'_>,
    default_context: Option<Context>,
    custom_dir: Option<PathBuf>,
) -> PyResult<Config> {
    // `None` → use the Python caller’s CWD (the same as the process CWD)
    let app_dir = custom_dir.unwrap_or_else(|| {
        // `current_dir` panics only if the process has no CWD
        let dir = env::current_dir().expect("Could not determine current working directory");
        dir.join(".eqty_sdk")
    });

    log::debug!("initializing sdk at {:?}", app_dir.display());
    let cfg = py.detach(|| get_runtime().block_on(Config::init(app_dir, default_context)))?;
    log::debug!("initialized at {:?}", cfg.app_dir);
    Ok(cfg)
}

/// Calculates and returns the CID for the provided bytes.
#[pyfunction]
#[pyo3(signature = (data, _store=None))]
fn get_cid_for_bytes(py: Python<'_>, data: &[u8], _store: Option<bool>) -> PyResult<CID> {
    with_cfg!(py, |ctx| {
        let cid = blake3_cid_raw_binary(data)?;
        let store_flag = _store.unwrap_or(ctx.store_all_blobs);

        if store_flag {
            ctx.blob_store
                .put(data.to_vec(), multicodec::RAW_BINARY, Some(&cid))
                .await?;
        }

        Ok(CID::new(cid))
    })
}

/// Calculates and returns the JCS CID for the provided JSON string.
#[pyfunction]
#[pyo3(signature = (json, _store=None))]
fn get_cid_for_json(py: Python<'_>, json: String, _store: Option<bool>) -> PyResult<CID> {
    with_cfg!(py, |ctx| {
        let json_value: Value = serde_json::from_str(&json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let (cid, data) = compute_jcs_cid(&json_value)?;
        let store_flag = _store.unwrap_or(ctx.store_all_blobs);

        if store_flag {
            ctx.blob_store
                .put(data.to_vec(), multicodec::JSON_JCS, Some(&cid))
                .await?;
        }

        Ok(CID::new(cid))
    })
}

/// Resolves the provided path and reads the file or directory to calculate the CID.
/// The path is saved to the blob store if the store flag is set
#[pyfunction]
#[pyo3(signature = (path, _store=None))]
fn get_cid_for_path(py: Python<'_>, path: PathBuf, _store: Option<bool>) -> PyResult<CID> {
    with_cfg!(py, |ctx| {
        let store_flag = _store.unwrap_or(ctx.store_all_blobs);

        if path.is_file() {
            let file_cid_result = compute_file_cid(path.clone(), ctx.hashing.clone()).await?;
            let cid = file_cid_result.cid.clone();

            if store_flag {
                let data = tokio::fs::read(&path).await?;
                ctx.blob_store
                    .put(data, multicodec::RAW_BINARY, Some(&cid))
                    .await?;
            }

            Ok(CID::new(cid))
        } else if path.is_dir() {
            let dir_cid_result =
                compute_dir_cid(path.clone(), ctx.hashing.clone(), ctx.cid_ignore.clone()).await?;
            let cid = dir_cid_result.collection.cid.clone();

            // Always store iroh collections
            let collection_codec = get_multicodec(&dir_cid_result.collection.cid)?;
            ctx.blob_store
                .put(
                    dir_cid_result.collection.blob.to_vec(),
                    collection_codec,
                    Some(&dir_cid_result.collection.cid),
                )
                .await?;
            let meta_codec = get_multicodec(&dir_cid_result.meta.cid)?;
            ctx.blob_store
                .put(
                    dir_cid_result.meta.blob.to_vec(),
                    meta_codec,
                    Some(&dir_cid_result.meta.cid),
                )
                .await?;

            if store_flag {
                for (file_name, file_cid) in dir_cid_result.file_hashes {
                    let src = path.join(file_name);
                    let data = fs::read(src).await?;
                    ctx.blob_store
                        .put(data, multicodec::RAW_BINARY, Some(&file_cid))
                        .await?;
                }
            }

            Ok(CID::new(cid))
        } else {
            Err(PyRuntimeError::new_err(format!(
                "The provided path {path:?} was not found"
            )))
        }
    })
}

/// Purges all statemetns from the store.
#[pyfunction]
#[pyo3(signature = ())]
fn purge_statement_store(py: Python<'_>) -> PyResult<()> {
    with_cfg!(py, |ctx| {
        ctx.sql_lite.purge().await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

/// Purges all blobs from the blob store.
#[pyfunction]
#[pyo3(signature = ())]
fn purge_blob_store(py: Python<'_>) -> PyResult<()> {
    with_cfg!(py, |ctx| {
        let blob_dir = ctx.app_dir.join("blobs");
        if blob_dir.exists() {
            tokio::fs::remove_dir_all(&blob_dir).await?;
            tokio::fs::create_dir_all(&blob_dir).await?;
        }
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pyo3::{
        exceptions::PyTypeError,
        types::{PyAnyMethods, PyDict, PyDictMethods, PyModuleMethods},
        wrap_pyfunction, Py, PyErr, Python,
    };
    use tempfile::tempdir;
    use uuid::Uuid;

    use crate::{config::Config, indexer::Context};

    #[test]
    fn test_init_accepts_default_context_positionally_and_custom_dir_as_keyword() {
        let temp_dir = tempdir().unwrap();
        Python::initialize();

        Python::attach(|py| {
            py.detach(|| {
                pyo3_async_runtimes::tokio::get_runtime()
                    .block_on(Config::reset_internal())
                    .unwrap();
            });

            let module = pyo3::types::PyModule::new(py, "_rust_test").unwrap();
            module
                .add_function(wrap_pyfunction!(crate::init, &module).unwrap())
                .unwrap();

            let default_context = Context {
                id: Uuid::new_v4(),
                name: "custom-default".to_string(),
                parent: None,
            };
            let py_context = Py::new(py, default_context.clone()).unwrap();
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("custom_dir", temp_dir.path().to_string_lossy().to_string())
                .unwrap();

            let cfg: Py<Config> = module
                .getattr("init")
                .unwrap()
                .call((py_context,), Some(&kwargs))
                .unwrap()
                .extract()
                .unwrap();

            assert_eq!(cfg.bind(py).borrow().default_context.id, default_context.id);

            py.detach(|| {
                pyo3_async_runtimes::tokio::get_runtime()
                    .block_on(Config::reset_internal())
                    .unwrap();
            });
        });
    }

    #[test]
    fn test_init_rejects_custom_dir_as_second_positional_argument() {
        let temp_dir = tempdir().unwrap();
        Python::initialize();

        Python::attach(|py| {
            py.detach(|| {
                pyo3_async_runtimes::tokio::get_runtime()
                    .block_on(Config::reset_internal())
                    .unwrap();
            });

            let module = pyo3::types::PyModule::new(py, "_rust_test").unwrap();
            module
                .add_function(wrap_pyfunction!(crate::init, &module).unwrap())
                .unwrap();

            let default_context = Context {
                id: Uuid::new_v4(),
                name: "custom-default".to_string(),
                parent: None,
            };
            let py_context = Py::new(py, default_context).unwrap();

            let err: PyErr = module
                .getattr("init")
                .unwrap()
                .call1((py_context, temp_dir.path().to_string_lossy().to_string()))
                .unwrap_err();

            assert!(err.is_instance_of::<PyTypeError>(py));

            py.detach(|| {
                pyo3_async_runtimes::tokio::get_runtime()
                    .block_on(Config::reset_internal())
                    .unwrap();
            });
        });
    }
}
