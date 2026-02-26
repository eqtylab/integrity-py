use bytes::Bytes;
use pyo3::{prelude::*, types::PyBytes, Bound};

/// Canonicalization algorithm for computing content identifiers.
#[derive(Clone, Copy, Debug)]
#[pyclass]
pub enum Canon {
    /// RDF Dataset Canonicalization 1.0 algorithm.
    RDFC1,
    /// JSON Canonicalization Scheme (JCS) algorithm.
    JSONJCS,
}

/// Result of computing a CID for a directory.
///
/// Contains the collection CID, metadata CID, and individual file hashes.
#[derive(Clone, Debug)]
#[pyclass]
pub struct DirCidResult {
    /// CID result for the directory collection.
    pub collection: CidResult,
    /// CID result for the directory metadata.
    pub meta: CidResult,
    /// List of tuples containing (file name, CID) for each file in the directory.
    pub file_hashes: Vec<(String, String)>,
}

impl From<integrity::cid::iroh::DirCidResult> for DirCidResult {
    fn from(value: integrity::cid::iroh::DirCidResult) -> Self {
        DirCidResult {
            collection: value.collection.into(),
            meta: value.meta.into(),
            file_hashes: value.file_hashes,
        }
    }
}

#[pymethods]
impl DirCidResult {
    #[getter]
    fn collection(&self) -> CidResult {
        self.collection.clone()
    }

    #[getter]
    fn meta(&self) -> CidResult {
        self.meta.clone()
    }

    #[getter]
    fn file_hashes(&self) -> Vec<(String, String)> {
        self.file_hashes.clone()
    }
}

/// Result of computing a content identifier (CID).
///
/// Contains both the CID string and the raw blob data.
#[derive(Clone, Debug)]
#[pyclass]
pub struct CidResult {
    /// The computed content identifier string.
    pub cid: String,
    /// The raw blob data used to compute the CID.
    pub blob: Bytes,
}

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

impl From<integrity::cid::iroh::CidResult> for CidResult {
    fn from(value: integrity::cid::iroh::CidResult) -> Self {
        CidResult {
            cid: value.cid,
            blob: value.blob,
        }
    }
}

#[pymethods]
impl CidResult {
    #[getter]
    fn cid(&self) -> String {
        self.cid.clone()
    }

    #[getter]
    fn blob<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(py, &self.blob))
    }
}

impl std::str::FromStr for Canon {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RDFC1" => Ok(Canon::RDFC1),
            "JSONJCS" => Ok(Canon::JSONJCS),
            _ => Err(()),
        }
    }
}

/// `cid` submodule.
#[pymodule]
pub fn cid(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Canon>()?;
    m.add_class::<DirCidResult>()?;
    m.add_class::<CidResult>()?;
    m.add_class::<CID>()?;

    Ok(())
}
