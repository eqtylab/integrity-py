//! EQTY SDK - Python bindings for the Integrity framework.
//!
//! This crate provides Python bindings via PyO3 for cryptographic signing,
//! content-addressed storage, and lineage statement management functionality.

/// Content identifier (CID) computation and utilities.
pub mod cid;
/// Global application context and configuration management.
pub mod context;
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

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(cid::cid))?;
    m.add_wrapped(wrap_pymodule!(context::context))?;
    m.add_wrapped(wrap_pymodule!(signer::signer))?;
    m.add_wrapped(wrap_pymodule!(manifest::manifest))?;
    m.add_wrapped(wrap_pymodule!(statements::statements))?;
    m.add_wrapped(wrap_pymodule!(stream::stream))?;

    m.add_function(wrap_pyfunction!(enable_rust_logging, m)?)?;

    Ok(())
}

#[pyfunction]
#[pyo3(signature = (is_unit_test = None), text_signature = "(is_unit_test: Optional[bool] = None) -> None")]
/// Enables log messages from the rust code. Log levels can be set through the RUST_LOG env var
/// os.environ['RUST_LOG'] = 'trace'
/// os.environ['RUST_LOG'] = 'integrity_core_rs::iroh=debug,api_utils::error=off,lineage=...'
fn enable_rust_logging(is_unit_test: Option<bool>) {
    // Use `try_init` to avoid panic if logger is initialized multiple times
    let _ = env_logger::builder()
        .is_test(is_unit_test.unwrap_or_default())
        .try_init();
}
