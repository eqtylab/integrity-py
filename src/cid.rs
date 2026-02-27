use pyo3::prelude::*;

/// A simple wrapper around a content identifier (CID) string.
///
/// Provides a typed wrapper for CID strings with property access and string conversion.
#[derive(Clone, Debug)]
#[pyclass]
pub struct CID {
    #[pyo3(get)]
    cid: String,
}

#[pymethods]
impl CID {
    #[new]
    fn new(cid: String) -> Self {
        CID { cid }
    }

    fn __str__(&self) -> String {
        self.cid.clone()
    }

    fn __repr__(&self) -> String {
        format!("Cid(\"{}\")", self.cid)
    }
}
