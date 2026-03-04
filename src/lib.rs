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
        multicodec,
    },
};
use pyo3::exceptions::PyRuntimeError;
use pyo3_async_runtimes::tokio::get_runtime;
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
/// Declaration model for governance statements.
pub mod declaration;
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

use pyo3::{prelude::*, wrap_pymodule};

use crate::{indexer::Graph, uuid::UUID};

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize pyo3-log to route Rust log messages to Python's logging module
    let _ = pyo3_log::try_init();

    m.add_wrapped(wrap_pymodule!(entity::entity))?;
    m.add_wrapped(wrap_pymodule!(signer::signer))?;
    m.add_wrapped(wrap_pymodule!(statements::statements))?;
    m.add_wrapped(wrap_pymodule!(stream::stream))?;

    m.add_class::<cid::CID>()?;
    m.add_class::<UUID>()?;
    m.add_class::<Graph>()?;
    m.add_class::<indexer::GraphFactory>()?;
    m.add_class::<Config>()?;
    m.add_class::<declaration::Declaration>()?;
    m.add_class::<did::DID>()?;
    m.add_class::<signer::Signer>()?;
    m.add_class::<signer::SignerAlgorithms>()?;
    m.add_class::<entity::Entity>()?;
    m.add_class::<did::DidFactory>()?;
    m.add_class::<integrity_service::Service>()?;
    m.add_class::<statements::PyAssociationType>()?;

    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_path, m)?)?;
    m.add_function(wrap_pyfunction!(purge_statement_store, m)?)?;
    m.add_function(wrap_pyfunction!(purge_blob_store, m)?)?;
    Ok(())
}

/// Initializes the sdk config. Must be called before setting individual config values
#[pyfunction]
#[pyo3(signature = (custom_dir=None, default_context=None))]
fn init(
    py: Python<'_>,
    custom_dir: Option<PathBuf>,
    default_context: Option<Graph>,
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
#[pyo3(signature = (data, store=None))]
fn get_cid_for_bytes(py: Python<'_>, data: &[u8], store: Option<bool>) -> PyResult<CID> {
    with_cfg!(py, |ctx| {
        let cid = blake3_cid_raw_binary(data)?;
        let store_flag = store.unwrap_or(ctx.store_all_blobs);

        if store_flag {
            ctx.blob_store
                .put(data.to_vec(), multicodec::RAW_BINARY, Some(&cid))
                .await?;
        }

        Ok(CID::new(cid))
    })
}

/// Resolves the provided path and reads the file or directory to calculate the CID.
/// The path is saved to the blob store if the store flag is set
#[pyfunction]
#[pyo3(signature = (path, store=None))]
fn get_cid_for_path(py: Python<'_>, path: PathBuf, store: Option<bool>) -> PyResult<CID> {
    with_cfg!(py, |ctx| {
        let store_flag = store.unwrap_or(ctx.store_all_blobs);

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
