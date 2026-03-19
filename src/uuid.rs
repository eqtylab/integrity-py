use std::fmt;

use pyo3::prelude::*;

/// A simple wrapper around a UUID string.
///
/// Provides a typed wrapper for UUID strings with property access and string conversion.
#[derive(Clone, Debug)]
#[pyclass]
pub struct UUID {
    /// The formated UUID string.
    #[pyo3(get)]
    uuid: String,
}

#[pymethods]
impl UUID {
    #[new]
    /// Creates a new UUID, ensuring it is prefixed with `urn:uuid:`.
    pub fn new(uuid: String) -> Self {
        let uuid = if uuid.starts_with("urn:uuid:") {
            uuid
        } else {
            format!("urn:uuid:{uuid}")
        };

        UUID { uuid }
    }

    fn __str__(&self) -> String {
        self.uuid.clone()
    }

    fn __repr__(&self) -> String {
        format!("UUID(\"{}\")", self.uuid)
    }

    fn __len__(&self) -> usize {
        self.uuid.len()
    }

    fn __getitem__(&self, index: isize) -> PyResult<String> {
        let chars: Vec<char> = self.uuid.chars().collect();
        let len = chars.len() as isize;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "UUID index out of range",
            ));
        }
        Ok(chars[idx as usize].to_string())
    }

    fn startswith(&self, prefix: &str) -> bool {
        self.uuid.starts_with(prefix)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl From<&str> for UUID {
    fn from(s: &str) -> Self {
        UUID::new(s.to_string())
    }
}

impl From<String> for UUID {
    fn from(s: String) -> Self {
        UUID::new(s)
    }
}
