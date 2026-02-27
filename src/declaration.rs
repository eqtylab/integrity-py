use std::collections::HashMap;

use integrity::{blob_store::BlobStore, cid::blake3::blake3_cid_raw_binary};
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyList},
    IntoPyObjectExt,
};
use pyo3_async_runtimes::tokio::get_runtime;
use serde_json::{Map, Value};

use crate::config::ctx_blocking;

/// A governance declaration describing a subject and related metadata.
#[pyclass]
#[derive(Clone)]
pub struct Declaration {
    /// Human-readable declaration subject line.
    #[pyo3(get, set)]
    pub subject_line: String,
    /// Declaration statement text.
    #[pyo3(get, set)]
    pub statement: String,
    /// ISO-8601 timestamp when the declaration was submitted.
    #[pyo3(get, set)]
    pub submitted_at: Option<String>,
    /// DID key of the signer who submitted the declaration.
    #[pyo3(get, set)]
    pub submitted_by: Option<String>,
    /// CIDs that are under the declarant's control.
    #[pyo3(get, set)]
    pub control_cid: Vec<String>,
    /// CIDs attached to this declaration.
    #[pyo3(get, set)]
    pub attachment_cid: Vec<String>,
    /// Additional key/value metadata for the declaration.
    #[pyo3(get, set)]
    pub extra: HashMap<String, String>,
}

#[pymethods]
impl Declaration {
    #[new]
    fn new(subject_line: String, statement: String) -> Self {
        Self {
            subject_line,
            statement,
            submitted_at: None,
            submitted_by: None,
            control_cid: Vec::new(),
            attachment_cid: Vec::new(),
            extra: HashMap::new(),
        }
    }

    #[staticmethod]
    #[pyo3(name = "new")]
    fn new_decl(subject_line: String, statement: String) -> Self {
        Self::new(subject_line, statement)
    }

    fn add_attachment_cid(&mut self, cid: String) -> PyResult<Self> {
        self.attachment_cid.push(cid);
        Ok(self.clone())
    }

    fn add_control_cid(&mut self, cid: String) -> PyResult<Self> {
        self.control_cid.push(cid);
        Ok(self.clone())
    }

    fn add_extra(&mut self, key: String, val: String) -> PyResult<Self> {
        self.extra.insert(key, val);
        Ok(self.clone())
    }

    fn finalize(&mut self) -> PyResult<Self> {
        let submitted_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let submitted_by = ctx_blocking()
            .and_then(|ctx| ctx.get_active_signer_did_key())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        self.submitted_at = Some(submitted_at);
        self.submitted_by = Some(submitted_by);

        let declaration_json = self.to_json()?;

        let blob_store = ctx_blocking()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
            .blob_store
            .clone();
        get_runtime()
            .block_on(blob_store.put(declaration_json.as_bytes().to_vec(), 0, None))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(self.clone())
    }

    fn cid(&self) -> PyResult<String> {
        let declaration_json = self.to_json()?;
        blake3_cid_raw_binary(declaration_json.as_bytes())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let value = self.to_value();
        json_value_to_python(py, &value)
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.to_value())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

impl Declaration {
    fn to_value(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "submittedAt".to_string(),
            self.submitted_at
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        map.insert(
            "submittedBy".to_string(),
            self.submitted_by
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        map.insert(
            "controlCid".to_string(),
            Value::Array(
                self.control_cid
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );

        if !self.subject_line.is_empty() {
            map.insert(
                "subjectLine".to_string(),
                Value::String(self.subject_line.clone()),
            );
        }
        if !self.statement.is_empty() {
            map.insert(
                "statement".to_string(),
                Value::String(self.statement.clone()),
            );
        }
        if !self.attachment_cid.is_empty() {
            map.insert(
                "attachmentCid".to_string(),
                Value::Array(
                    self.attachment_cid
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
        if !self.extra.is_empty() {
            let extra_map = self
                .extra
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            map.insert("extra".to_string(), Value::Object(extra_map));
        }

        Value::Object(map)
    }
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
