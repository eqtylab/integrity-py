//! EQTY SDK - Python bindings for the Integrity framework.
//!
//! This crate provides Python bindings via PyO3 for cryptographic signing,
//! content-addressed storage, and lineage statement management functionality.

use config::Config;
use pyo3_async_runtimes::tokio::get_runtime;
use std::env;
use std::path::PathBuf;

/// Content identifier (CID) computation and utilities.
pub mod cid;
/// Global application configuration management.
pub mod config;
/// Indexes integrity information in sql database.
pub mod indexer;
/// API functions for connecting to the integrity service.
pub mod integrity_service;
/// Manifest generation, import, and registration for integrity graphs.
pub mod manifest;
/// Cryptographic signer creation and management.
pub mod signer;
/// Lineage statement creation and storage operations.
pub mod statements;
/// Streaming computation support for real-time data processing.
pub mod stream;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

use crate::indexer::Graph;

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize pyo3-log to route Rust log messages to Python's logging module
    let _ = pyo3_log::try_init();

    m.add_wrapped(wrap_pymodule!(cid::cid))?;
    m.add_wrapped(wrap_pymodule!(signer::signer))?;
    m.add_wrapped(wrap_pymodule!(manifest::manifest))?;
    m.add_wrapped(wrap_pymodule!(statements::statements))?;
    m.add_wrapped(wrap_pymodule!(stream::stream))?;

    m.add_class::<Graph>()?;
    m.add_class::<Config>()?;

    m.add_function(wrap_pyfunction!(init, m)?)?;
    Ok(())
}

/// Initializes the sdk config. Must be called before setting individual config values
#[pyfunction]
#[pyo3(signature = (app_dir=None))]
fn init(py: Python<'_>, app_dir: Option<PathBuf>) -> PyResult<Config> {
    // `None` → use the Python caller’s CWD (the same as the process CWD)
    let app_dir = app_dir.unwrap_or_else(|| {
        // `current_dir` panics only if the process has no CWD
        let dir = env::current_dir().expect("Could not determine current working directory");
        dir.join(".eqty_sdk")
    });

    let cfg = py.detach(|| get_runtime().block_on(Config::init(app_dir)))?;
    log::debug!("initialized at {:?}", cfg.app_dir);
    Ok(cfg)
}
