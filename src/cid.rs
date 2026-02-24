use std::path::PathBuf;

use bytes::Bytes;
use integrity::cid::{
    blake3::blake3_cid_raw_binary,
    iroh::{compute_dir_cid, compute_file_cid},
};
use pyo3::{prelude::*, types::PyBytes, Bound};

use crate::with_ctx;

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
    m.add_function(wrap_pyfunction!(compute_cid_for_directory, m)?)?;
    m.add_function(wrap_pyfunction!(compute_cid_for_file, m)?)?;
    m.add_function(wrap_pyfunction!(compute_cid_for_bytes, m)?)?;
    m.add_class::<Canon>()?;
    m.add_class::<DirCidResult>()?;
    m.add_class::<CidResult>()?;
    m.add_class::<CID>()?;

    Ok(())
}

/// Compute CID for a directory at `path`.
#[pyfunction]
#[pyo3(signature = (path), text_signature = "(path: PathLike) -> DirCidResult")]
fn compute_cid_for_directory(py: Python, path: PathBuf) -> PyResult<DirCidResult> {
    with_ctx!(py, |ctx| {
        let dir_cid_result =
            compute_dir_cid(path.clone(), ctx.hashing.clone(), ctx.cid_ignore.clone()).await?;

        Ok(dir_cid_result.into())
    })
}

/// Compute CID for a file `path`.
#[pyfunction]
#[pyo3(signature = (path), text_signature = "(path: PathLike) -> CidResult")]
fn compute_cid_for_file(py: Python, path: PathBuf) -> PyResult<CidResult> {
    with_ctx!(py, |ctx| {
        Ok(compute_file_cid(path.clone(), ctx.hashing).await?.into())
    })
}

/// Compute CID for provided bytes.
#[pyfunction]
#[pyo3(signature = (bytes), text_signature = "(bytes: bytes) -> str")]
fn compute_cid_for_bytes(_py: Python, bytes: &[u8]) -> PyResult<String> {
    Ok(blake3_cid_raw_binary(bytes)?)
}
