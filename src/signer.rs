use std::fs;

use anyhow::Context as AnyhowContext;
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::signer::{
    load_signer as utils_load_signer, save_signer as utils_save_signer, AuthServiceSigner,
    Ed25519Signer, KeyType, P256Signer, Secp256k1Signer, SignerType, VCompNotarySigner,
    YubiHsmSigner,
};
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
    types::PyAny,
    Bound,
};
use serde::Serialize;

use crate::{config::ctx_blocking, with_ctx, Config};

/// `signer` submodule.
#[pymodule]
pub fn signer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Signer>()?;
    m.add_class::<SignerAlgorithms>()?;
    m.add_function(wrap_pyfunction!(create_new_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_signer_from_private_key, m)?)?;
    m.add_function(wrap_pyfunction!(create_vcomp_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_yubihsm2_signer, m)?)?;
    m.add_function(wrap_pyfunction!(create_auth_service_signer, m)?)?;
    m.add_function(wrap_pyfunction!(set_active_signer, m)?)?;

    Ok(())
}

/// Python-exposed signer information.
///
/// Contains the name and DID key of a cryptographic signer.
#[pyclass(name = "Signer")]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signer {
    /// Human-readable name for the signer.
    pub name: String,
    /// Decentralized Identifier (DID) key for the signer.
    pub did_key: String,
}

#[pyclass(name = "SIGNER_ALGORITHMS")]
pub struct SignerAlgorithms;

#[pymethods]
impl SignerAlgorithms {
    #[classattr]
    const ED25519: &'static str = "ed25519";
    #[classattr]
    const SECP256K1: &'static str = "secp256k1";
    #[classattr]
    const SECP256R1: &'static str = "secp256r1";
}

#[pymethods]
impl Signer {
    #[new]
    fn py_new(name: String, did_key: String) -> Self {
        Self { name, did_key }
    }

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

    #[staticmethod]
    #[pyo3(name = "new", signature = (algorithm=None))]
    fn new_signer(py: Python, algorithm: Option<&Bound<'_, PyAny>>) -> PyResult<Py<Signer>> {
        let key_type = signer_algorithm_from_py(algorithm)?;
        create_new_signer_internal(py, key_type, None)
    }

    #[staticmethod]
    #[pyo3(signature = (url=None))]
    fn vcomp_notary(py: Python, url: Option<String>) -> PyResult<Py<Signer>> {
        let url = url.unwrap_or_else(|| "http://docker.eqtylab.internal:8066".to_string());
        create_vcomp_signer(py, url, None)
    }

    #[staticmethod]
    #[pyo3(signature = (url))]
    fn auth_service(py: Python, url: String) -> PyResult<Py<Signer>> {
        let api_key = std::env::var("EQTY_API_KEY").map_err(|_| {
            PyRuntimeError::new_err(
                "The env var 'EQTY_API_KEY' must be set to use Signer.auth_service()",
            )
        })?;
        create_auth_service_signer(py, url, api_key)
    }

    #[staticmethod]
    #[pyo3(signature = (auth_key_id, signing_key_id, password))]
    fn yubihsm2(
        py: Python,
        auth_key_id: u16,
        signing_key_id: u16,
        password: String,
    ) -> PyResult<Py<Signer>> {
        create_yubihsm2_signer(py, auth_key_id, signing_key_id, password)
    }

    #[staticmethod]
    #[pyo3(signature = (algorithm, private_key))]
    fn from_private_key(
        py: Python,
        algorithm: &Bound<'_, PyAny>,
        private_key: String,
    ) -> PyResult<Py<Signer>> {
        let key_type = signer_algorithm_from_py(Some(algorithm))?;
        create_signer_from_private_key_internal(py, key_type, private_key, None)
    }
}

/// Creates a new local signer with a randomly generated key.
///
/// # Arguments
/// * `name` - Optional name for the signer (uses DID key if not provided)
/// * `key_type` - Type of cryptographic key to generate (SECP256K1, SECP256R1, ED25519)
#[pyfunction]
#[pyo3(signature = (key_type, name=None))]
fn create_new_signer(py: Python, key_type: String, name: Option<&str>) -> PyResult<Py<Signer>> {
    signer_exists(name)?;

    let key_type: KeyType = key_type.parse().context("Invalid key type")?;
    create_new_signer_internal(py, key_type, name)
}

fn create_new_signer_internal(
    py: Python,
    key_type: KeyType,
    name: Option<&str>,
) -> PyResult<Py<Signer>> {
    let signer = match key_type {
        KeyType::SECP256K1 => {
            log::trace!("Generating a new secp256k1 signer");
            let signer = Secp256k1Signer::create()?;
            SignerType::SECP256K1(signer)
        }
        KeyType::SECP256R1 => {
            log::trace!("Generating a new secp256r1 signer");
            let signer = P256Signer::create()?;
            SignerType::P256(signer)
        }
        KeyType::ED25519 => {
            log::trace!("Generating a new ed25519 signer");
            let signer = Ed25519Signer::create()?;
            SignerType::ED25519(signer)
        }
    };

    let signer = save_signer(&signer, name)?;
    Py::new(py, signer)
}

/// Creates a signer from an existing base64-encoded private key.
///
/// # Arguments
/// * `key` - Base64-encoded private key bytes
/// * `key_type` - Type of cryptographic key (SECP256K1, SECP256R1, ED25519)
/// * `name` - Optional name for the signer (uses DID key if not provided)
#[pyfunction]
#[pyo3(signature = (key, key_type, name=None))]
fn create_signer_from_private_key(
    py: Python,
    key: String,
    key_type: String,
    name: Option<&str>,
) -> PyResult<Py<Signer>> {
    signer_exists(name)?;

    let key_type: KeyType = key_type.parse().context("Invalid key type")?;
    create_signer_from_private_key_internal(py, key_type, key, name)
}

fn create_signer_from_private_key_internal(
    py: Python,
    key_type: KeyType,
    key: String,
    name: Option<&str>,
) -> PyResult<Py<Signer>> {
    log::info!("Creating a signer of type '{key_type}'");

    let secret_key = BASE64
        .decode(key.as_bytes())
        .context("Invalid base64 key")?;

    let signer = match key_type {
        KeyType::SECP256R1 => {
            log::trace!("Creating a P256 signer from a private key.");
            let signer = P256Signer::import(&secret_key)?;
            SignerType::P256(signer)
        }
        KeyType::SECP256K1 => {
            log::trace!("Creating a SECP256K1 signer from a private key.");
            let signer = Secp256k1Signer::import(&secret_key)?;
            SignerType::SECP256K1(signer)
        }
        KeyType::ED25519 => {
            log::trace!("Creating a ED25519 signer from a private key.");
            let signer = Ed25519Signer::import(&secret_key)?;
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
fn create_vcomp_signer(py: Python, url: String, pub_key: Option<String>) -> PyResult<Py<Signer>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let signer = rt.block_on(VCompNotarySigner::create(&url, pub_key))?;

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
fn create_auth_service_signer(py: Python, url: String, api_key: String) -> PyResult<Py<Signer>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    log::debug!("Creating auth service signer {url:?}");

    let signer = rt.block_on(AuthServiceSigner::create(api_key, url))?;

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
) -> PyResult<Py<Signer>> {
    log::trace!("Importing a YubiHSM2 ed25519 signer");

    let yubi_signer = YubiHsmSigner::create(auth_key_id, signing_key_id, password)?;

    let signer_type = SignerType::YubiHsm2Signer(yubi_signer);
    let signer = save_signer(&signer_type, None)?;
    Py::new(py, signer)
}

/// Sets the active signer by name or signer instance.
///
/// # Arguments
/// * `signer` - Signer name string or Signer instance
#[pyfunction]
#[pyo3(signature = (signer), text_signature = "(signer: str | Signer) -> None")]
fn set_active_signer(py: Python, signer: &Bound<'_, PyAny>) -> PyResult<()> {
    let name = if let Ok(name) = signer.extract::<String>() {
        name
    } else if let Ok(name_attr) = signer.getattr("name") {
        name_attr.extract::<String>()?
    } else {
        return Err(PyErr::new::<PyTypeError, _>(
            "signer must be a signer name string or a Signer instance",
        ));
    };
    with_ctx!(py, |ctx| {
        log::debug!("Setting '{name}' as the active");
        let signer_file = ctx.app_dir.join(SIGNER_DIR).join(&name);
        if !signer_file.exists() {
            return Err(anyhow::anyhow!("No Signer named '{name}' found"));
        }

        let signer = utils_load_signer(signer_file)?;
        Config::set_active_signer_async(signer).await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

/// Subdirectory name for storing signer key files.
pub static SIGNER_DIR: &str = "signers";

/// Checks if a signer already exists with the provided name
fn signer_exists(name: Option<&str>) -> PyResult<()> {
    if name.is_none() {
        return Ok(());
    }

    let signer_dir = ctx_blocking()?.app_dir.join(SIGNER_DIR);
    let name = name.unwrap();
    log::debug!("Adding Signer. Args= {name}");

    if fs::exists(signer_dir.join(name)).expect("Error checking if signer exists") {
        let msg = format!("A signer named {name:?} already exists");
        log::warn!("{msg}");
        return Err(PyErr::new::<PyValueError, _>(msg));
    }

    Ok(())
}

fn save_signer(signer: &SignerType, name: Option<&str>) -> PyResult<Signer> {
    let did_key = signer.get_did_doc().id;
    let name = name.unwrap_or(&did_key);
    let signer_dir = ctx_blocking()?.app_dir.join(SIGNER_DIR);
    fs::create_dir_all(signer_dir.clone())?;
    utils_save_signer(signer, signer_dir, name)?;
    Ok(Signer {
        name: name.to_owned(),
        did_key,
    })
}

fn signer_algorithm_from_py(obj: Option<&Bound<'_, PyAny>>) -> PyResult<KeyType> {
    let key_type = if let Some(obj) = obj {
        if let Ok(s) = obj.extract::<String>() {
            s
        } else if let Ok(value) = obj.getattr("value") {
            value.extract::<String>()?
        } else {
            return Err(PyErr::new::<PyTypeError, _>(
                "algorithm must be a string or enum with a 'value' attribute",
            ));
        }
    } else {
        "ed25519".to_string()
    };

    key_type
        .parse()
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("{e}")))
}
