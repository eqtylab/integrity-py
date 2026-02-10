use integrity::lineage::models::statements::Statement;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
    IntoPyObjectExt,
};

use crate::with_ctx;

mod association;
mod computation;
mod data;
mod did;
mod entity;
mod governance;
mod metadata;
mod model_signing;
/// Storage statement creation for referencing external data stores.
pub mod storage;
mod vc;

use anyhow::Context as AnyhowContext;
use uuid::Uuid;

/// `statements` submodule to create lineage statements
#[pymodule]
pub fn statements(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(
        association::create_association_statement,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        computation::create_computation_statement,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(data::add_data_statement, m)?)?;
    m.add_function(wrap_pyfunction!(data::create_data_statement, m)?)?;
    m.add_function(wrap_pyfunction!(did::create_did_statement, m)?)?;
    m.add_function(wrap_pyfunction!(entity::create_entity_statement, m)?)?;
    m.add_function(wrap_pyfunction!(
        governance::create_governance_statement,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(metadata::create_metadata_statement, m)?)?;
    m.add_function(wrap_pyfunction!(vc::create_vc_statement, m)?)?;

    m.add_function(wrap_pyfunction!(storage::create_storage_statement, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_graph, m)?)?;
    m.add_function(wrap_pyfunction!(register_statement, m)?)?;
    m.add_function(wrap_pyfunction!(register_statement_to_graph, m)?)?;

    m.add_function(wrap_pyfunction!(
        model_signing::create_model_signing_statement,
        m
    )?)?;

    Ok(())
}

/// Retrieve statements for multiple graph IDs.
///
/// Args:
///     graph_ids: List of graph UUIDs to retrieve graphs for
///
/// Returns:
///     List of statements
#[pyfunction]
#[pyo3(signature = (graph_ids), text_signature = "(graph_ids: list[UUID]) -> list[Statements]")]
pub fn retrieve_graph(py: Python, graph_ids: Vec<Uuid>) -> PyResult<Py<PyList>> {
    let statements: Vec<Statement> = with_ctx!(py, |ctx| {
        let sql_client = ctx.sql_lite;

        log::info!("Retrieving graphs {graph_ids:?}");

        let mut statements: Vec<Statement> = Vec::new();
        for graph_id in graph_ids.clone() {
            let graph_statements = sql_client.retrieve_statements(&graph_id).await?;
            statements.extend(graph_statements);
        }
        Ok::<_, anyhow::Error>(statements)
    })?;

    // Convert statements to Python objects
    let py_statements: Vec<Py<PyAny>> = statements
        .into_iter()
        .map(|stmt| {
            let value = serde_json::to_value(&stmt).context("Failed to serialize statement")?;
            json_value_to_python(py, &value)
        })
        .collect::<PyResult<Vec<_>>>()?;

    Ok(PyList::new(py, py_statements)?.unbind())
}

/// Register a statement from JSON string to the default graph.
#[pyfunction]
#[pyo3(signature = (statement_json))]
pub fn register_statement(py: Python, statement_json: String) -> PyResult<()> {
    with_ctx!(py, |ctx| {
        let statement: Statement = serde_json::from_str(&statement_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse statement JSON: {}", e))?;
        let graph_id = ctx.default_graph.id;
        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

/// Associate an existing statement with a graph.
#[pyfunction]
#[pyo3(signature = (statement_id, graph_id))]
pub fn register_statement_to_graph(
    py: Python,
    statement_id: String,
    graph_id: String,
) -> PyResult<()> {
    with_ctx!(py, |ctx| {
        let graph_uuid =
            Uuid::parse_str(&graph_id).map_err(|e| anyhow::anyhow!("Invalid graph UUID: {}", e))?;
        ctx.sql_lite
            .associate_statement_to_graph(&statement_id, &graph_uuid)
            .await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

fn json_value_to_python(py: Python, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    use serde_json::Value;
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => b.into_py_any(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py)
            } else {
                n.to_string().into_py_any(py)
            }
        }
        Value::String(s) => s.into_py_any(py),
        Value::Array(arr) => {
            let py_list: PyResult<Vec<Py<PyAny>>> =
                arr.iter().map(|v| json_value_to_python(py, v)).collect();
            PyList::new(py, py_list?)?.into_py_any(py)
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, json_value_to_python(py, v)?)?;
            }
            py_dict.into_py_any(py)
        }
    }
}
