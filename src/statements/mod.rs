use crate::indexer::Graph;
use pyo3::{
    pyfunction,
    types::{PyDict, PyList},
    PyObject, PyResult, Python, ToPyObject,
};

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

use pyo3::prelude::*;
use pyo3::{pymodule, wrap_pyfunction};
use uuid::Uuid;

use crate::{
    context::{self, ctx},
    to_py_err,
};

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
pub fn retrieve_graph(py: Python, graph_ids: Vec<String>) -> PyResult<PyObject> {
    let sql_client = ctx().sql_lite;

    log::info!("Retrieving graphs {graph_ids:?}");

    let mut graphs: Vec<Graph> = Vec::new();
    for graph_id in graph_ids.clone() {
        let graph_uuid = Uuid::parse_str(&graph_id).map_err(to_py_err)?;
        let graph = context::get_runtime()
            .block_on(sql_client.retrieve_graph(&graph_uuid))
            .map_err(to_py_err)?;

        graphs.push(graph);
    }

    // Convert graphs to Python objects
    let py_graphs: Vec<PyObject> = graphs
        .into_iter()
        .map(|graph| {
            // Convert each graph to a Python dict
            let py_dict = PyDict::new_bound(py);

            // Set graph metadata
            py_dict.set_item("id", graph.id.to_string())?;
            py_dict.set_item("name", graph.name)?;
            py_dict.set_item("parent", graph.parent.map(|p| p.to_string()))?;

            // Convert statements to Python objects
            let py_statements: Vec<PyObject> = graph
                .statements
                .unwrap_or_default()
                .into_iter()
                .map(|stmt| {
                    let value = serde_json::to_value(&stmt).map_err(to_py_err)?;
                    json_value_to_python(py, &value)
                })
                .collect::<PyResult<Vec<_>>>()?;

            py_dict.set_item("statements", PyList::new_bound(py, py_statements))?;

            Ok(py_dict.into())
        })
        .collect::<PyResult<Vec<_>>>()?;

    Ok(PyList::new_bound(py, py_graphs).into())
}

fn json_value_to_python(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    use serde_json::Value;
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Ok(n.to_string().to_object(py))
            }
        }
        Value::String(s) => Ok(s.to_object(py)),
        Value::Array(arr) => {
            let py_list: PyResult<Vec<PyObject>> =
                arr.iter().map(|v| json_value_to_python(py, v)).collect();
            Ok(PyList::new_bound(py, py_list?).to_object(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new_bound(py);
            for (k, v) in obj {
                py_dict.set_item(k, json_value_to_python(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}
