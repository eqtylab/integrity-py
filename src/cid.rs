use std::fmt;

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
    /// Creates a new CID, ensuring it is prefixed with `urn:cid:`.
    pub fn new(cid: String) -> Self {
        let cid = if cid.starts_with("urn:cid:") {
            cid
        } else {
            format!("urn:cid:{cid}")
        };

        CID { cid }
    }

    fn __str__(&self) -> String {
        self.cid.clone()
    }

    fn __repr__(&self) -> String {
        format!("Cid(\"{}\")", self.cid)
    }

    fn __len__(&self) -> usize {
        self.cid.len()
    }

    fn __getitem__(&self, index: isize) -> PyResult<String> {
        let chars: Vec<char> = self.cid.chars().collect();
        let len = chars.len() as isize;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "CID index out of range",
            ));
        }
        Ok(chars[idx as usize].to_string())
    }

    fn startswith(&self, prefix: &str) -> bool {
        self.cid.starts_with(prefix)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.cid == other.cid
    }
}

impl fmt::Display for CID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cid)
    }
}

impl From<&str> for CID {
    fn from(s: &str) -> Self {
        CID::new(s.to_string())
    }
}

impl From<String> for CID {
    fn from(s: String) -> Self {
        CID::new(s)
    }
}
