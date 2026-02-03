use crate::indexer::Graph;
use crate::with_ctx;
use pyo3::types::{PyDict, PyList};
use pyo3::{prelude::*, IntoPyObjectExt};

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

    m.add_function(wrap_pyfunction!(
        model_signing::create_model_signing_statement,
        m
    )?)?;

    Ok(())
}

/// Retrieve graphs for multiple graph IDs.
///
/// Args:
///     graph_ids: List of graph ID strings to retrieve graphs for
///
/// Returns:
///     List of graph objects with their statements
#[pyfunction]
#[pyo3(signature = (graph_ids), text_signature = "(graph_ids: list[str]) -> list[dict]")]
pub fn retrieve_graph(py: Python, graph_ids: Vec<String>) -> PyResult<Py<PyList>> {
    let graphs: Vec<Graph> = with_ctx!(py, |ctx| {
        let sql_client = ctx.sql_lite;

        log::info!("Retrieving graphs {graph_ids:?}");

        let mut graphs: Vec<Graph> = Vec::new();
        for graph_id in graph_ids.clone() {
            let graph_uuid = Uuid::parse_str(&graph_id).context("Invalid graph ID")?;
            let graph = sql_client.retrieve_graph(&graph_uuid).await?;

            graphs.push(graph);
        }
        Ok::<_, anyhow::Error>(graphs)
    })?;

    // Convert graphs to Python objects
    let py_graphs: Vec<Py<PyAny>> = graphs
        .into_iter()
        .map(|graph| {
            // Convert each graph to a Python dict
            let py_dict = PyDict::new(py);

            // Set graph metadata
            py_dict.set_item("id", graph.id.to_string())?;
            py_dict.set_item("name", graph.name)?;
            py_dict.set_item("parent", graph.parent.map(|p| p.to_string()))?;

            // Convert statements to Python objects
            let py_statements: Vec<Py<PyAny>> = graph
                .statements
                .unwrap_or_default()
                .into_iter()
                .map(|stmt| {
                    let value =
                        serde_json::to_value(&stmt).context("Failed to serialize statement")?;
                    json_value_to_python(py, &value)
                })
                .collect::<PyResult<Vec<_>>>()?;

            py_dict.set_item("statements", PyList::new(py, py_statements)?)?;

            Ok(py_dict.into())
        })
        .collect::<PyResult<Vec<_>>>()?;

    Ok(PyList::new(py, py_graphs)?.unbind())
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
