//! EQTY SDK - Python bindings for the Integrity framework.
//!
//! This crate provides Python bindings via PyO3 for cryptographic signing,
//! content-addressed storage, and lineage statement management functionality.

use std::{env, path::PathBuf};

use config::{ctx_blocking, Config};
use integrity::cid::{
    blake3::blake3_cid_raw_binary,
    iroh::{compute_dir_cid, compute_file_cid},
};
use pyo3::exceptions::PyRuntimeError;
use pyo3_async_runtimes::tokio::get_runtime;
use tokio::fs;

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

/// Asset model and helpers.
pub mod asset;
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
/// Manifest generation, import, and registration for integrity graphs.
pub mod manifest;
/// Metadata model for subject descriptions.
pub mod metadata;
/// Cryptographic signer creation and management.
pub mod signer;
/// Lineage statement creation and storage operations.
pub mod statements;
/// Streaming computation support for real-time data processing.
pub mod stream;

use pyo3::{prelude::*, wrap_pymodule};

use crate::indexer::Graph;

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize pyo3-log to route Rust log messages to Python's logging module
    let _ = pyo3_log::try_init();

    m.add_wrapped(wrap_pymodule!(cid::cid))?;
    m.add_wrapped(wrap_pymodule!(entity::entity))?;
    m.add_wrapped(wrap_pymodule!(signer::signer))?;
    m.add_wrapped(wrap_pymodule!(manifest::manifest))?;
    m.add_wrapped(wrap_pymodule!(statements::statements))?;
    m.add_wrapped(wrap_pymodule!(stream::stream))?;

    m.add_class::<asset::Asset>()?;
    m.add_class::<cid::Canon>()?;
    m.add_class::<cid::Cid>()?;
    m.add_class::<cid::CidResult>()?;
    m.add_class::<cid::DirCidResult>()?;
    m.add_class::<Graph>()?;
    m.add_class::<Config>()?;
    m.add_class::<declaration::Declaration>()?;
    m.add_class::<did::Did>()?;
    m.add_class::<metadata::Metadata>()?;
    m.add_class::<manifest::Manifest>()?;
    m.add_class::<signer::Signer>()?;
    m.add_class::<signer::SignerAlgorithms>()?;
    m.add_class::<entity::Entity>()?;
    m.add_class::<did::DidFactory>()?;

    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(get_cid_for_path, m)?)?;
    m.add_function(wrap_pyfunction!(maybe_create_model_signing_statement, m)?)?;
    Ok(())
}

/// Initializes the sdk config. Must be called before setting individual config values
#[pyfunction]
#[pyo3(signature = (custom_dir=None))]
fn init(py: Python<'_>, custom_dir: Option<PathBuf>) -> PyResult<Config> {
    // `None` → use the Python caller’s CWD (the same as the process CWD)
    let app_dir = custom_dir.unwrap_or_else(|| {
        // `current_dir` panics only if the process has no CWD
        let dir = env::current_dir().expect("Could not determine current working directory");
        dir.join(".eqty_sdk")
    });

    let cfg = py.detach(|| get_runtime().block_on(Config::init(app_dir)))?;
    log::debug!("initialized at {:?}", cfg.app_dir);
    Ok(cfg)
}

/// Calculates and returns the CID for the provided bytes.
#[pyfunction]
#[pyo3(signature = (data, store=None))]
fn get_cid_for_bytes(py: Python<'_>, data: &[u8], store: Option<bool>) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let cid = blake3_cid_raw_binary(data)?;
        let store_flag = store.unwrap_or(ctx.store_all_blobs);

        if store_flag {
            let blob_dir = ctx.app_dir.join("blobs");
            fs::create_dir_all(&blob_dir).await?;
            let file_path = blob_dir.join(&cid);
            fs::write(&file_path, data).await?;
        }

        Ok(cid)
    })
}

/// Resolves the provided path and reads the file or directory to calculate the CID.
#[pyfunction]
#[pyo3(signature = (path, store=None))]
fn get_cid_for_path(py: Python<'_>, path: PathBuf, store: Option<bool>) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let store_flag = store.unwrap_or(ctx.store_all_blobs);
        let blob_dir = ctx.app_dir.join("blobs");
        fs::create_dir_all(&blob_dir).await?;

        if path.is_file() {
            let file_cid_result = compute_file_cid(path.clone(), ctx.hashing.clone()).await?;
            let cid = file_cid_result.cid.clone();

            if store_flag {
                let storage_path = blob_dir.join(&cid);
                fs::copy(&path, &storage_path).await?;
            }

            Ok(cid)
        } else if path.is_dir() {
            let dir_cid_result =
                compute_dir_cid(path.clone(), ctx.hashing.clone(), ctx.cid_ignore.clone()).await?;
            let cid = dir_cid_result.collection.cid.clone();

            // Always store iroh collections
            fs::write(
                blob_dir.join(&dir_cid_result.collection.cid),
                dir_cid_result.collection.blob,
            )
            .await?;
            fs::write(
                blob_dir.join(&dir_cid_result.meta.cid),
                dir_cid_result.meta.blob,
            )
            .await?;

            if store_flag {
                for (file_name, file_cid) in dir_cid_result.file_hashes {
                    let src = path.join(file_name);
                    let dst = blob_dir.join(file_cid);
                    fs::copy(src, dst).await?;
                }
            }

            Ok(cid)
        } else {
            Err(PyRuntimeError::new_err(format!(
                "The provided path {path:?} was not found"
            )))
        }
    })
}

/// Creates a model signing statement if enabled in config and the asset is a directory.
#[pyfunction]
#[pyo3(signature = (collection_cid, model_signing_name, is_dir))]
fn maybe_create_model_signing_statement(
    py: Python<'_>,
    collection_cid: String,
    model_signing_name: String,
    is_dir: bool,
) -> PyResult<()> {
    if !is_dir {
        return Ok(());
    }

    let ctx = ctx_blocking().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    if !ctx.generate_model_signing_signatures {
        return Ok(());
    }

    let blobs_dir = ctx.app_dir.join("blobs");
    std::fs::create_dir_all(&blobs_dir)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

    let allow_symlinks = ctx.cid_ignore.include_symlinks;
    crate::statements::model_signing::create_model_signing_statement(
        py,
        collection_cid,
        blobs_dir,
        model_signing_name,
        allow_symlinks,
        Vec::new(),
        None,
        None,
    )?;

    Ok(())
}
