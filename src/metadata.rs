use pyo3::{
    exceptions::PyKeyError,
    prelude::*,
    types::{PyAny, PyDict, PyIterator},
    IntoPyObjectExt,
};

use crate::statements;

#[pyclass]
pub struct Metadata {
    additional: Py<PyDict>,
}

#[pymethods]
impl Metadata {
    #[new]
    #[pyo3(signature = (**kwargs))]
    fn new(py: Python, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let dict = PyDict::new(py);
        if let Some(kwargs) = kwargs {
            for (k, v) in kwargs.iter() {
                let key: String = k.extract()?;
                if key == "skip_proof" {
                    continue;
                }
                dict.set_item(k, v)?;
            }
        }

        Ok(Self {
            additional: dict.unbind(),
        })
    }

    fn __getattr__(&self, py: Python, attr: &str) -> PyResult<Py<PyAny>> {
        let dict = self.additional.bind(py);
        if dict.contains(attr)? {
            if let Some(value) = dict.get_item(attr)? {
                return value.into_py_any(py);
            }
        }
        Ok(py.None())
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.additional.clone_ref(py).into_py_any(py)
    }

    fn to_json_str(&self, py: Python) -> PyResult<String> {
        let json = py.import("json")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("indent", 4)?;
        json.getattr("dumps")?
            .call((self.additional.bind(py),), Some(&kwargs))?
            .extract::<String>()
    }

    fn create_statement(
        &self,
        py: Python,
        subject_cid: String,
        skip_proof: bool,
    ) -> PyResult<Vec<String>> {
        let metadata_json = self.to_json_str(py)?;
        statements::metadata::add_metadata_statement(
            py,
            subject_cid,
            metadata_json,
            Some(skip_proof),
            None,
        )
    }

    fn __iter__(&self, py: Python) -> PyResult<Py<PyAny>> {
        let keys = self.additional.bind(py).keys();
        PyIterator::from_object(&keys)?.into_py_any(py)
    }

    fn __contains__(&self, py: Python, key: Py<PyAny>) -> PyResult<bool> {
        self.additional.bind(py).contains(key)
    }

    fn __getitem__(&self, py: Python, key: Py<PyAny>) -> PyResult<Py<PyAny>> {
        if let Some(value) = self.additional.bind(py).get_item(&key)? {
            return value.into_py_any(py);
        }
        let key_repr = key.bind(py).repr()?.extract::<String>()?;
        Err(PyKeyError::new_err(format!(
            "'{}' object has no attribute {:?}",
            self.additional.bind(py).get_type().name()?,
            key_repr
        )))
    }
}

impl Metadata {
    pub fn from_dict(dict: Bound<'_, PyDict>) -> Self {
        Self {
            additional: dict.unbind(),
        }
    }
}
