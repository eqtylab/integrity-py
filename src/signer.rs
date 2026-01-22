use std::{collections::HashMap, fs, path::PathBuf};

use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::signer::{
    load_signer as utils_load_signer, save_signer as utils_save_signer, AuthServiceSigner,
    Ed25519Signer, KeyType, P256Signer, Secp256k1Signer, SignerType, VCompNotarySigner,
    YubiHsmSigner,
};
use pyo3::{
    pyclass, pyfunction, pymethods, pymodule,
    types::{PyBytes, PyModule},
    wrap_pyfunction, Py, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use serde::Serialize;

use crate::{context::ctx, to_py_err};

/// `signer` submodule.
#[pymodule]
pub fn signer(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = py;

    m.add_function(wrap_pyfunction!(create_new_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_signer_from_private_key, m)?)?;
    m.add_function(wrap_pyfunction!(create_vcomp_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_yubihsm2_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_auth_service_signer, m)?)?;
    m.add_function(wrap_pyfunction!(set_active_signer, m)?)?;
    m.add_function(wrap_pyfunction!(get_active_signer_did_key, m)?)?;
    m.add_function(wrap_pyfunction!(get_signer_type, m)?)?;
    m.add_function(wrap_pyfunction!(get_signer_statements, m)?)?;
    m.add_function(wrap_pyfunction!(get_signer_blobs, m)?)?;

    Ok(())
}

/// Python-exposed signer information.
///
/// Contains the name and DID key of a cryptographic signer.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PySigner {
    /// Human-readable name for the signer.
    pub name: String,
    /// Decentralized Identifier (DID) key for the signer.
    pub did_key: String,
}

impl ToPyObject for PySigner {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("name", &self.name).unwrap();
        dict.set_item("did_key", &self.did_key).unwrap();
        dict.into()
    }
}
#[pymethods]
impl PySigner {
    /// Returns the human-readable name of the signer.
    ///
    /// # Returns
    /// * `&str` - The signer's name
    #[getter]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the DID key of the signer.
    ///
    /// # Returns
    /// * `&str` - The signer's DID key string
    #[getter]
    pub fn did_key(&self) -> &str {
        &self.did_key
    }
}

/// Creates a new local signer with a randomly generated key.
///
/// # Arguments
/// * `name` - Optional name for the signer (uses DID key if not provided)
/// * `key_type` - Type of cryptographic key to generate (SECP256K1, SECP256R1, ED25519)
#[pyfunction]
fn create_new_signer(py: Python, key_type: String, name: Option<&str>) -> PyResult<Py<PySigner>> {
    signer_exists(name)?;

    let key_type: KeyType = key_type.parse().map_err(to_py_err)?;

    let signer = match key_type {
        KeyType::SECP256K1 => {
            log::trace!("Generating a new secp256k1 signer");
            let signer = Secp256k1Signer::create().map_err(to_py_err)?;
            SignerType::SECP256K1(signer)
        }
        KeyType::SECP256R1 => {
            log::trace!("Generating a new secp256r1 signer");
            let signer = P256Signer::create().map_err(to_py_err)?;
            SignerType::P256(signer)
        }
        KeyType::ED25519 => {
            log::trace!("Generating a new ed25519 signer");
            let signer = Ed25519Signer::create().map_err(to_py_err)?;
            SignerType::ED25519(signer)
        }
    };

    let signer = save_signer(&signer, name)?;
    Py::new(py, signer)
}

/// Creates a signer from an existing base64-encoded private key.
///
/// # Arguments
/// * `name` - Optional name for the signer (uses DID key if not provided)
/// * `key` - Base64-encoded private key bytes
/// * `key_type` - Type of cryptographic key (SECP256K1, SECP256R1, ED25519)
#[pyfunction]
fn create_signer_from_private_key(
    py: Python,
    key: String,
    key_type: String,
    name: Option<&str>,
) -> PyResult<Py<PySigner>> {
    signer_exists(name)?;

    let key_type: KeyType = key_type.parse().map_err(to_py_err)?;

    log::info!("Creating a signer of type '{key_type}'");

    let secret_key = BASE64.decode(key.as_bytes()).map_err(to_py_err)?;

    let signer = match key_type {
        KeyType::SECP256R1 => {
            log::trace!("Creating a P256 signer from a private key.");
            let signer = P256Signer::import(&secret_key).map_err(to_py_err)?;
            SignerType::P256(signer)
        }
        KeyType::SECP256K1 => {
            log::trace!("Creating a SECP256K1 signer from a private key.");
            let signer = Secp256k1Signer::import(&secret_key).map_err(to_py_err)?;
            SignerType::SECP256K1(signer)
        }
        KeyType::ED25519 => {
            log::trace!("Creating a ED25519 signer from a private key.");
            let signer = Ed25519Signer::import(&secret_key).map_err(to_py_err)?;
            SignerType::ED25519(signer)
        }
    };
    let signer = save_signer(&signer, name)?;
    Py::new(py, signer)
}

/// Creates a VComp notary signer for TEE-based remote signing.
///
/// # Arguments
/// * `name` - Name to assign to the signer
/// * `url` - VComp notary service URL
/// * `key_type` - Type of key (currently only SECP256R1 is supported)
/// * `pub_key` - Optional public key for the signer
///
#[pyfunction]
fn create_vcomp_signer(py: Python, url: String, pub_key: Option<String>) -> PyResult<Py<PySigner>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let signer = rt
        .block_on(VCompNotarySigner::create(&url, pub_key))
        .map_err(to_py_err)?;

    let signer_type = SignerType::VCompNotarySigner(signer);
    let signer = save_signer(&signer_type, None)?;
    Py::new(py, signer)
}

/// Creates an Auth Service-based signer for remote signing operations.
///
/// # Arguments
/// * `name` - Name to assign to the signer
/// * `url` -  Auth Service API endpoint URL
/// * `api_key` - API key for authentication with the Auth Service
#[pyfunction]
fn create_auth_service_signer(py: Python, url: String, api_key: String) -> PyResult<Py<PySigner>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    log::debug!("Creating auth service signer {url:?}");

    let signer = rt
        .block_on(AuthServiceSigner::create(api_key, url))
        .map_err(to_py_err)?;

    let signer_type = SignerType::AuthService(signer);
    let signer = save_signer(&signer_type, None)?;
    Py::new(py, signer)
}

/// Creates and configures a YubiHSM2 hardware security module signer.
///
/// # Arguments
/// * `name` - Name to assign to the signer
/// * `auth_key_id` - Authentication key ID for YubiHSM2
/// * `signing_key_id` - Signing key ID for YubiHSM2
/// * `password` - Password for YubiHSM2 authentication
#[pyfunction]
fn create_yubihsm2_signer(
    py: Python,
    auth_key_id: u16,
    signing_key_id: u16,
    password: String,
) -> PyResult<Py<PySigner>> {
    log::trace!("Importing a YubiHSM2 ed25519 signer");

    let yubi_signer =
        YubiHsmSigner::create(auth_key_id, signing_key_id, password).map_err(to_py_err)?;

    let signer_type = SignerType::YubiHsm2Signer(yubi_signer);
    let signer = save_signer(&signer_type, None)?;
    Py::new(py, signer)
}

/// Sets the active signer by name.
///
/// # Arguments
/// * `name` - Name of the signer to make active
#[pyfunction]
#[pyo3(signature = (name), text_signature = "(name: str) -> None")]
fn set_active_signer(_py: Python, name: String) -> PyResult<()> {
    log::debug!("Setting '{name}' as the active");
    let signer_file = get_signer_folder().join(name);
    if !signer_file.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No Signer named '{name}' found",
        ));
    }

    let signer = utils_load_signer(signer_file).map_err(to_py_err)?;
    ctx().set_active_signer(signer).map_err(to_py_err)
}

/// Get the active signers Did Key
#[pyfunction]
fn get_active_signer_did_key(_py: Python) -> PyResult<String> {
    ctx().get_active_signer_did_key().map_err(to_py_err)
}

/// Get signer type string ('vcomp_notary', 'yubihsm2', etc) by name.
///
/// # Arguments
/// * `name` - Name of the signer to retrieve
#[pyfunction]
fn get_signer_type(_py: Python, name: String) -> PyResult<String> {
    let signer_file = get_signer_folder().join(name);
    if !signer_file.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No Signer named '{name}' found",
        ));
    }

    let signer = utils_load_signer(signer_file).map_err(to_py_err)?;
    let signer_type = format!("{signer}");

    Ok(signer_type)
}

/// Retrieves the statements associated with a signer if any.
///
/// # Arguments
/// * `name` - Name of the signer to retrieve statements from
#[pyfunction]
fn get_signer_statements(_py: Python, name: String) -> PyResult<Vec<String>> {
    let signer_file = get_signer_folder().join(name);
    if !signer_file.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No Signer named '{name}' found",
        ));
    }

    let signer = utils_load_signer(signer_file).map_err(to_py_err)?;

    match signer {
        SignerType::VCompNotarySigner(vcomp_signer) => {
            if let Some(statements) = vcomp_signer.did_statements {
                let statements = statements
                    .values()
                    .cloned()
                    .map(|v| serde_json::to_string(&v))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(to_py_err)?;
                Ok(statements)
            } else {
                Ok(vec![])
            }
        }
        _ => Ok(vec![]),
    }
}

/// Retrieves the blobs associated with a signer if any.
///
/// # Arguments
/// * `name` - Name of the signer to retrieve blobs from
#[pyfunction]
fn get_signer_blobs(py: Python<'_>, name: String) -> PyResult<HashMap<String, &PyBytes>> {
    let signer_file = get_signer_folder().join(name);
    if !signer_file.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No Signer named '{name}' found",
        ));
    }

    let signer = utils_load_signer(signer_file).map_err(to_py_err)?;

    match signer {
        SignerType::VCompNotarySigner(vcomp_signer) => {
            if let Some(blobs) = vcomp_signer.did_blobs {
                let blobs = blobs
                    .into_iter()
                    .map(|(k, v)| (k, PyBytes::new(py, &v)))
                    .collect();
                Ok(blobs)
            } else {
                Ok(HashMap::new())
            }
        }
        _ => Ok(HashMap::new()),
    }
}

/// Checks if a signer already exists with the provided name
fn signer_exists(name: Option<&str>) -> PyResult<()> {
    if name.is_none() {
        return Ok(());
    }

    let signer_folder = get_signer_folder();
    let name = name.unwrap();
    log::debug!("Adding Signer. Args= {name}");

    if fs::exists(signer_folder.join(name)).expect("Error checking if signer exists") {
        let msg = format!("A signer named {name:?} already exists");
        log::warn!("{msg}");
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg));
    }

    Ok(())
}

fn save_signer(signer: &SignerType, name: Option<&str>) -> PyResult<PySigner> {
    let did_key = signer.get_did_doc().id;
    let name = name.unwrap_or(&did_key);
    let signer_dir = get_signer_folder();
    fs::create_dir_all(signer_dir.clone()).map_err(to_py_err)?;
    utils_save_signer(signer, signer_dir, name).map_err(to_py_err)?;
    Ok(PySigner {
        name: name.to_owned(),
        did_key,
    })
}

/// Subdirectory name for storing signer key files.
static SIGNER_DIR: &str = "signers";

/// Returns the path to the signer storage folder.
///
/// # Returns
/// * `PathBuf` - Path to the directory where signer key files are stored
pub fn get_signer_folder() -> PathBuf {
    ctx().app_dir.join(SIGNER_DIR)
}
