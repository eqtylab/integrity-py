use std::path::PathBuf;

use bytes::Bytes;
use integrity::cid::{
    blake3::blake3_cid_raw_binary,
    iroh::{compute_dir_cid, compute_file_cid},
};
use pyo3::{
    pyclass, pyfunction, pymethods, pymodule,
    types::{PyBytes, PyModule},
    wrap_pyfunction, PyResult, Python,
};

use crate::{
    context::{self, ctx},
    to_py_err,
};

#[derive(Clone, Copy, Debug)]
#[pyclass]
pub enum Canon {
    RDFC1,
    JSONJCS,
}

#[derive(Clone, Debug)]
#[pyclass]
pub struct DirCidResult {
    pub collection: CidResult,
    pub meta: CidResult,
    pub file_hashes: Vec<(String, String)>, // (File Name, CID)
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

#[derive(Clone, Debug)]
#[pyclass]
pub struct CidResult {
    pub cid: String,
    pub blob: Bytes,
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
    fn blob<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
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
pub fn cid(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = py;

    m.add_function(wrap_pyfunction!(compute_cid_for_directory, m)?)?;
    m.add_function(wrap_pyfunction!(compute_cid_for_file, m)?)?;
    m.add_function(wrap_pyfunction!(compute_cid_for_bytes, m)?)?;
    m.add_class::<Canon>()?;
    m.add_class::<DirCidResult>()?;
    m.add_class::<CidResult>()?;

    Ok(())
}

/// Compute CID for a directory at `path`.
#[pyfunction]
#[pyo3(signature = (path), text_signature = "(path: PathLike) -> DirCidResult")]
fn compute_cid_for_directory(_py: Python, path: PathBuf) -> PyResult<DirCidResult> {
    let ctx = ctx();

    let dir_cid_result = context::get_runtime()
        .block_on(compute_dir_cid(
            path.clone(),
            ctx.hashing.clone(),
            ctx.cid_ignore.clone(),
        ))
        .map_err(to_py_err)?;

    Ok(dir_cid_result.into())
}

/// Compute CID for a file `path`.
#[pyfunction]
#[pyo3(signature = (path), text_signature = "(path: PathLike) -> CidResult")]
fn compute_cid_for_file(_py: Python, path: PathBuf) -> PyResult<CidResult> {
    let context = ctx();

    Ok(context::get_runtime()
        .block_on(compute_file_cid(path.clone(), context.hashing))
        .map_err(to_py_err)?
        .into())
}

/// Compute CID for provided bytes.
#[pyfunction]
#[pyo3(signature = (bytes), text_signature = "(bytes: bytes) -> str")]
fn compute_cid_for_bytes(_py: Python, bytes: &[u8]) -> PyResult<String> {
    blake3_cid_raw_binary(bytes).map_err(to_py_err)
}
