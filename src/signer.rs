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

use crate::{config::cfg_blocking, with_cfg, Config};

/// `signer` submodule.
#[pymodule]
pub fn signer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Signer>()?;
    m.add_class::<SignerAlgorithms>()?;
    m.add_function(wrap_pyfunction!(set_active_signer, m)?)?;
    m.add_function(wrap_pyfunction!(get_active_signer_did_key, m)?)?;

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

/// Supported signer algorithm identifiers.
#[pyclass(name = "SIGNER_ALGORITHMS")]
#[derive(Clone)]
pub enum SignerAlgorithms {
    /// Ed25519 signature algorithm.
    ED25519,
    /// Secp256k1 signature algorithm.
    SECP256K1,
    /// Secp256r1 (P-256) signature algorithm.
    SECP256R1,
}

impl From<SignerAlgorithms> for KeyType {
    fn from(value: SignerAlgorithms) -> Self {
        match value {
            SignerAlgorithms::ED25519 => KeyType::ED25519,
            SignerAlgorithms::SECP256K1 => KeyType::SECP256K1,
            SignerAlgorithms::SECP256R1 => KeyType::SECP256R1,
        }
    }
}

impl From<KeyType> for SignerAlgorithms {
    fn from(value: KeyType) -> Self {
        match value {
            KeyType::ED25519 => SignerAlgorithms::ED25519,
            KeyType::SECP256K1 => SignerAlgorithms::SECP256K1,
            KeyType::SECP256R1 => SignerAlgorithms::SECP256R1,
        }
    }
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

    /// Creates a local signer and persists it to disk.
    ///
    /// If `name` is provided, the signer is stored under that name. When
    /// `_load_if_exists=True`, an existing signer with the same name is loaded
    /// instead of creating a new key. If no algorithm is provided, Ed25519 is
    /// used.
    #[staticmethod]
    #[pyo3(
        name = "new",
        signature = (algorithm=None, name=None, _load_if_exists=false),
        text_signature = "(algorithm=None, name=None, _load_if_exists=False)"
    )]
    fn new_signer(
        py: Python,
        algorithm: Option<SignerAlgorithms>,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        if let Some(existing) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        let signer = match algorithm.unwrap_or(SignerAlgorithms::ED25519).into() {
            KeyType::SECP256K1 => {
                log::debug!("Generating a new secp256k1 signer");
                let signer = Secp256k1Signer::create()?;
                SignerType::SECP256K1(signer)
            }
            KeyType::SECP256R1 => {
                log::debug!("Generating a new secp256r1 signer");
                let signer = P256Signer::create()?;
                SignerType::P256(signer)
            }
            KeyType::ED25519 => {
                log::debug!("Generating a new ed25519 signer");
                let signer = Ed25519Signer::create()?;
                SignerType::ED25519(signer)
            }
        };

        let signer = save_signer(&signer, name.as_deref())?;
        Py::new(py, signer)
    }

    /// Creates a VComp notary signer and persists it to disk.
    ///
    /// If `name` is provided, the signer is stored under that name. When
    /// `_load_if_exists=True`, an existing signer with the same name is loaded
    /// instead of creating a new remote signer configuration.
    #[staticmethod]
    #[pyo3(signature = (url=None, name=None, _load_if_exists=false))]
    fn vcomp_notary(
        py: Python,
        url: Option<String>,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        let url = url.unwrap_or_else(|| "http://docker.eqtylab.internal:8066".to_string());
        if let Some(existing) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let signer = rt.block_on(VCompNotarySigner::create(&url, None))?;

        let signer_type = SignerType::VCompNotarySigner(signer);
        let signer = save_signer(&signer_type, name.as_deref())?;
        Py::new(py, signer)
    }

    /// Creates an Auth Service signer and persists it to disk.
    ///
    /// Requires the `EQTY_API_KEY` environment variable to be set. If `name`
    /// is provided, the signer is stored under that name. When
    /// `_load_if_exists=True`, an existing signer with the same name is loaded
    /// instead of creating a new remote signer configuration.
    #[staticmethod]
    #[pyo3(signature = (url, name=None, _load_if_exists=false))]
    fn auth_service(
        py: Python,
        url: String,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        let api_key = std::env::var("EQTY_API_KEY").map_err(|_| {
            PyRuntimeError::new_err(
                "The env var 'EQTY_API_KEY' must be set to use Signer.auth_service()",
            )
        })?;

        if let Some(existing) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        log::debug!("Creating auth service signer {url:?}");

        let signer = rt.block_on(AuthServiceSigner::create(api_key, url))?;

        let signer_type = SignerType::AuthService(signer);
        let signer = save_signer(&signer_type, name.as_deref())?;
        Py::new(py, signer)
    }

    /// Creates a YubiHSM2-backed signer and persists it to disk.
    ///
    /// If `name` is provided, the signer is stored under that name. When
    /// `_load_if_exists=True`, an existing signer with the same name is loaded
    /// instead of creating a new hardware-backed signer configuration.
    #[staticmethod]
    #[pyo3(signature = (auth_key_id, signing_key_id, password, name=None, _load_if_exists=false))]
    fn yubihsm2(
        py: Python,
        auth_key_id: u16,
        signing_key_id: u16,
        password: String,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        if let Some(existing) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        log::debug!("Importing a YubiHSM2 ed25519 signer");

        let yubi_signer = YubiHsmSigner::create(auth_key_id, signing_key_id, password)?;

        let signer_type = SignerType::YubiHsm2Signer(yubi_signer);
        let signer = save_signer(&signer_type, name.as_deref())?;
        Py::new(py, signer)
    }

    /// Creates a signer from a base64-encoded private key and persists it.
    ///
    /// If `name` is provided, the signer is stored under that name. When
    /// `_load_if_exists=True`, an existing signer with the same name is loaded
    /// instead of importing the provided private key.
    #[staticmethod]
    #[pyo3(
        signature = (algorithm, private_key, name=None, _load_if_exists=false),
        text_signature = "(algorithm, private_key, name=None, _load_if_exists=False)"
    )]
    fn from_private_key(
        py: Python,
        algorithm: SignerAlgorithms,
        private_key: String,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        if let Some(existing) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        let key_type: KeyType = algorithm.into();
        log::info!("Creating a signer of type '{:?}'", key_type);

        let secret_key = BASE64
            .decode(private_key.as_bytes())
            .context("Invalid base64 key")?;

        let signer = match key_type {
            KeyType::SECP256R1 => {
                log::debug!("Creating a P256 signer from a private key.");
                let signer = P256Signer::import(&secret_key)?;
                SignerType::P256(signer)
            }
            KeyType::SECP256K1 => {
                log::debug!("Creating a SECP256K1 signer from a private key.");
                let signer = Secp256k1Signer::import(&secret_key)?;
                SignerType::SECP256K1(signer)
            }
            KeyType::ED25519 => {
                log::debug!("Creating a ED25519 signer from a private key.");
                let signer = Ed25519Signer::import(&secret_key)?;
                SignerType::ED25519(signer)
            }
        };
        let signer = save_signer(&signer, name.as_deref())?;
        Py::new(py, signer)
    }
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
    with_cfg!(py, |ctx| {
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

/// Returns the DID key of the currently active signer.
#[pyfunction]
fn get_active_signer_did_key() -> PyResult<String> {
    cfg_blocking()
        .and_then(|ctx| ctx.get_active_signer_did_key())
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Subdirectory name for storing signer key files.
pub static SIGNER_DIR: &str = "signers";

/// Checks if a signer already exists with the provided name
fn signer_exists(name: Option<&str>) -> PyResult<()> {
    if name.is_none() {
        return Ok(());
    }

    let signer_dir = cfg_blocking()?.app_dir.join(SIGNER_DIR);
    let name = name.unwrap();
    log::debug!("Adding Signer. Args= {name}");

    if fs::exists(signer_dir.join(name)).expect("Error checking if signer exists") {
        let msg = format!("A signer named {name:?} already exists");
        log::warn!("{msg}");
        return Err(PyErr::new::<PyValueError, _>(msg));
    }

    Ok(())
}

fn maybe_load_signer(name: Option<&str>, load_if_exists: bool) -> PyResult<Option<Signer>> {
    if !load_if_exists {
        signer_exists(name)?;
        return Ok(None);
    }

    let Some(name) = name else {
        return Err(PyErr::new::<PyValueError, _>(
            "_load_if_exists=True requires a signer name",
        ));
    };

    let signer_path = cfg_blocking()?.app_dir.join(SIGNER_DIR).join(name);
    if !signer_path.exists() {
        return Ok(None);
    }

    let signer = utils_load_signer(signer_path)?;
    Ok(Some(Signer {
        name: name.to_owned(),
        did_key: signer.get_did_doc().id,
    }))
}

fn save_signer(signer: &SignerType, name: Option<&str>) -> PyResult<Signer> {
    let did_key = signer.get_did_doc().id;
    let name = name.unwrap_or(&did_key);
    let signer_dir = cfg_blocking()?.app_dir.join(SIGNER_DIR);
    fs::create_dir_all(signer_dir.clone())?;
    utils_save_signer(signer, signer_dir, name)?;
    Ok(Signer {
        name: name.to_owned(),
        did_key,
    })
}
