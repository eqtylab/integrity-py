use integrity::lineage::{
    graph_indexer::sql_indexer::IStatementIdx as graphIdx, indexer::sql_indexer::IStatementIdx,
    models::graph::Graph,
};
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
pub mod storage;
mod vc;

use pyo3::{pymodule, types::PyModule, wrap_pyfunction, PyErr};
use uuid::Uuid;

use crate::{
    context::{self, ctx},
    convert_attributes,
    feature_flags::FeatureFlags,
    to_py_err,
};

/// `statements` submodule to create lineage statements
#[pymodule]
pub fn statements(_py: Python, m: &PyModule) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(register_statement_locally, m)?)?;
    m.add_function(wrap_pyfunction!(add_attributes_to_statements, m)?)?;
    m.add_function(wrap_pyfunction!(remove_attributes, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_statements, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_graph, m)?)?;
    m.add_function(wrap_pyfunction!(delete_statements, m)?)?;

    m.add_function(wrap_pyfunction!(
        model_signing::create_model_signing_statement,
        m
    )?)?;

    m.add_function(wrap_pyfunction!(register_statement_to_graph, m)?)?;

    Ok(())
}

/// Register a statement locally in the database.
///
/// Args:
///     statement: JSON string representation of the statement to register
#[pyfunction]
#[pyo3(signature = (statement), text_signature = "(statement: str) -> None")]
pub fn register_statement_locally(_py: Python, statement: String) -> PyResult<()> {
    let statement = serde_json::from_str(&statement).map_err(to_py_err)?;

    context::get_runtime()
        .block_on(ctx().register_statement_locally(statement, None, None))
        .map_err(to_py_err)?;

    Ok(())
}

/// Register a statement to a specific graph.
///
/// Args:
///     statement_id: The statement ID to register
///     graph_id: The graph ID to register the statement to
#[pyfunction]
#[pyo3(signature = (statement_id, graph_id), text_signature = "(statement_id: str, graph_id: str) -> None")]
pub fn register_statement_to_graph(
    _py: Python,
    statement_id: String,
    graph_id: String,
) -> PyResult<()> {
    let feature_flag = "graph_ids";
    if FeatureFlags::is_enabled(feature_flag) {
        let graph_id = Uuid::parse_str(&graph_id).map_err(to_py_err)?;
        let sql_client = ctx().sql_lite2;
        context::get_runtime()
            .block_on(sql_client.associate_statement_to_graph(&statement_id, &graph_id))
            .map_err(to_py_err)?;

        Ok(())
    } else {
        let msg = format!("Feature {feature_flag} must be disabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    }
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
    let feature_flag = "graph_ids";

    if FeatureFlags::is_enabled(feature_flag) {
        let sql_client = ctx().sql_lite2;

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
                let py_dict = PyDict::new(py);

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

                py_dict.set_item("statements", PyList::new(py, py_statements))?;

                Ok(py_dict.into())
            })
            .collect::<PyResult<Vec<_>>>()?;

        Ok(PyList::new(py, py_graphs).into())
    } else {
        let msg = format!("Feature {feature_flag} must be enabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    }
}

/// Retrieve statements using a filter query (legacy mode when graph_ids feature is disabled).
///
/// Args:
///     filter_query: Optional SQL filter query string
///
/// Returns:
///     Tuple of (statements list, attributes dict)
#[pyfunction]
#[pyo3(signature = (filter_query = None), text_signature = "(filter_query: str | None = None) -> tuple[list[dict], dict]")]
pub fn retrieve_statements(
    py: Python,
    filter_query: Option<String>,
) -> PyResult<(PyObject, PyObject)> {
    let feature_flag = "graph_ids";

    if FeatureFlags::is_enabled(feature_flag) {
        let msg = format!("Feature {feature_flag} must be disabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    } else {
        log::info!(
            "Retrieving statements with filter query: {}",
            filter_query.clone().unwrap_or("None".to_owned())
        );
        let sql_client = ctx().sql_lite;
        let (statements, attributes) = context::get_runtime()
            .block_on(sql_client.retrieve_statements(filter_query.as_deref()))
            .map_err(to_py_err)?;
        log::info!("Found {} matching statements", statements.len());

        // Convert statements to Python objects
        let py_statements: Vec<PyObject> = statements
            .into_iter()
            .map(|stmt| {
                // Convert Statement to serde_json::Value, then to Python
                let value = serde_json::to_value(&stmt).map_err(to_py_err)?;
                json_value_to_python(py, &value)
            })
            .collect::<PyResult<Vec<_>>>()?;

        let attributes_value = serde_json::to_value(&attributes).map_err(to_py_err)?;
        let py_attributes = json_value_to_python(py, &attributes_value)?;
        Ok((PyList::new(py, py_statements).into(), py_attributes))
    }
}

/// Add attributes to statements (legacy mode when graph_ids feature is disabled).
///
/// Args:
///     statement_cids: List of statement CID strings to tag
///     attributes: Dictionary of attributes to add
#[pyfunction]
#[pyo3(signature = (statement_cids, attributes), text_signature = "(statement_cids: list[str], attributes: dict) -> None")]
pub fn add_attributes_to_statements(
    _py: Python,
    statement_cids: Vec<String>,
    attributes: &PyDict,
) -> PyResult<()> {
    let feature_flag = "graph_ids";

    if FeatureFlags::is_enabled(feature_flag) {
        let msg = format!("Feature {feature_flag} must be disabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    } else {
        let attributes = convert_attributes(attributes).map_err(to_py_err)?;
        let attributes = serde_json::to_value(attributes).map_err(to_py_err)?;

        log::info!("Tagging statements {statement_cids:?} with attributes: {attributes:?}");
        let sql_client = ctx().sql_lite;
        context::get_runtime()
            .block_on(sql_client.update_statements_attributes(&statement_cids, &attributes))
            .map_err(to_py_err)?;

        Ok(())
    }
}

/// Remove attributes from statements (legacy mode when graph_ids feature is disabled).
///
/// Args:
///     statement_cids: List of statement CID strings to remove attributes from
///     attributes: Dictionary of attributes to remove
#[pyfunction]
#[pyo3(signature = (statement_cids, attributes), text_signature = "(statement_cids: list[str], attributes: dict) -> None")]
pub fn remove_attributes(
    _py: Python,
    statement_cids: Vec<String>,
    attributes: &PyDict,
) -> PyResult<()> {
    let feature_flag = "graph_ids";

    if FeatureFlags::is_enabled(feature_flag) {
        let msg = format!("Feature {feature_flag} must be disabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    } else {
        log::info!("Removing attributes {attributes:?} from statements {statement_cids:?} ");
        let attributes = convert_attributes(attributes).map_err(to_py_err)?;
        let attributes = serde_json::to_value(attributes).map_err(to_py_err)?;
        let sql_client = ctx().sql_lite;
        context::get_runtime()
            .block_on(sql_client.remove_attributes(&statement_cids, &attributes))
            .map_err(to_py_err)?;
        Ok(())
    }
}

/// Delete statements using a filter query (legacy mode when graph_ids feature is disabled).
///
/// Args:
///     filter_query: Optional SQL filter query string to specify which statements to delete
#[pyfunction]
#[pyo3(signature = (filter_query = None), text_signature = "(filter_query: str | None = None) -> None")]
pub fn delete_statements(_py: Python, filter_query: Option<String>) -> PyResult<()> {
    let feature_flag = "graph_ids";

    if FeatureFlags::is_enabled(feature_flag) {
        let msg = format!("Feature {feature_flag} must be disabled to use this fn.");
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
    } else {
        log::info!(
            "Deleting statements with filter query: {}",
            filter_query.clone().unwrap_or("None".to_owned())
        );
        let sql_client = ctx().sql_lite;
        context::get_runtime()
            .block_on(sql_client.delete_statements(filter_query.as_deref()))
            .map_err(to_py_err)?;

        Ok(())
    }
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
            Ok(PyList::new(py, py_list?).to_object(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, json_value_to_python(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}
