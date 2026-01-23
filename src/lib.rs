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

use core::fmt::Display;
use std::collections::HashMap;

use anyhow::Result;
use pyo3::{
    exceptions::PyRuntimeError,
    pyfunction, pymodule,
    types::{PyDict, PyModule},
    wrap_pyfunction, wrap_pymodule, PyErr, PyResult, Python,
};
use serde_json::{json, Value};

/// SDK rust module
///
/// This module is accessible in the Python package as `eqty_sdk._rust`
#[pymodule]
fn _rust(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = py;

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
// Helper fn to reduce boilerplate error handling
fn to_py_err<E: Display>(error: E) -> PyErr {
    PyRuntimeError::new_err(format!("{error}"))
}

/// Converts a Python dictionary to a Rust HashMap of JSON values.
///
/// # Arguments
/// * `attributes` - Python dictionary containing key-value pairs to convert
///
/// # Returns
/// * `Result<HashMap<String, Value>>` - HashMap with string keys and JSON values,
///   or an error if conversion fails
pub fn convert_attributes(attributes: &PyDict) -> Result<HashMap<String, Value>> {
    let mut attr_map: HashMap<String, Value> = HashMap::new();
    for (key, value) in attributes {
        let key_str: String = key.extract()?;
        // Handle different Python value types
        let converted_value = if let Ok(i) = value.extract::<i64>() {
            json!(i)
        } else if let Ok(u) = value.extract::<u64>() {
            json!(u)
        } else if let Ok(f) = value.extract::<f64>() {
            json!(f)
        } else {
            json!(value.to_string())
        };

        attr_map.insert(key_str, converted_value);
    }
    Ok(attr_map)
}
