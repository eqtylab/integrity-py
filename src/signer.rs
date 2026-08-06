use std::{ffi::CString, fs};

use anyhow::Context as AnyhowContext;
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use integrity::{
    blob_store::BlobStore,
    cid::get_multicodec,
    lineage::models::statements::Statement,
    signer::{
        load_signer as utils_load_signer, save_signer as utils_save_signer, AuthServiceSigner,
        Ed25519Signer, KeyType, P256Signer, Secp256k1Signer, SignerType, VCompNotarySigner,
    },
};
use pyo3::{
    exceptions::{PyDeprecationWarning, PyLookupError, PyRuntimeError, PyValueError},
    prelude::*,
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
#[pyclass(name = "Signer", from_py_object)]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signer {
    /// Human-readable name for the signer.
    pub name: String,
    /// Decentralized Identifier (DID) key for the signer.
    pub did_key: String,
}

/// Supported signer algorithm identifiers.
#[pyclass(name = "SIGNER_ALGORITHMS", from_py_object)]
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

    /// Creates a new local signer and persists it to disk.
    ///
    /// Use this when you want the SDK to generate a fresh signing key for the
    /// current workflow. The returned signer can be passed to
    /// `set_active_signer(...)` so higher-level SDK operations emit signed
    /// statements and attestations automatically.
    ///
    /// If `name` is provided, the signer is stored under that name; creating a
    /// second signer under a name that already exists raises `ValueError`. To
    /// reuse a persisted signer instead, use `Signer.load_or_create(...)`. If no
    /// algorithm is provided, Ed25519 is used.
    #[staticmethod]
    #[pyo3(
        name = "new",
        signature = (algorithm=None, name=None, _load_if_exists=false),
        text_signature = "(algorithm=None, name=None)"
    )]
    fn new_signer(
        py: Python,
        algorithm: Option<SignerAlgorithms>,
        name: Option<String>,
        _load_if_exists: bool,
    ) -> PyResult<Py<Signer>> {
        if _load_if_exists {
            warn_load_if_exists_deprecated(py, "Signer.load_or_create(name=...)")?;
        }

        if let Some((existing, _)) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
            return Py::new(py, existing);
        }

        let signer = create_local_signer(algorithm)?;
        let signer = save_signer(&signer, name.as_deref())?;
        Py::new(py, signer)
    }

    /// Loads a signer that was previously persisted under `name`.
    ///
    /// Raises `LookupError` if no such signer exists. Use
    /// `Signer.load_or_create(...)` to create one when it is missing.
    ///
    /// This works for any persisted signer regardless of how it was created,
    /// including `auth_service` and `vcomp_notary` signers.
    #[staticmethod]
    #[pyo3(signature = (name), text_signature = "(name)")]
    fn load(py: Python, name: String) -> PyResult<Py<Signer>> {
        match maybe_load_signer(Some(&name), true)? {
            Some((existing, _)) => Py::new(py, existing),
            None => Err(PyErr::new::<PyLookupError, _>(format!(
                "No signer named {name:?} exists"
            ))),
        }
    }

    /// Loads the signer named `name`, generating and persisting one if it does
    /// not exist yet.
    ///
    /// This is idempotent, so it is the right call for a script that runs more
    /// than once: the first run generates a key, later runs reuse it and keep a
    /// stable DID. If no algorithm is provided, Ed25519 is used. The algorithm
    /// is ignored when an existing signer is loaded.
    #[staticmethod]
    #[pyo3(signature = (name, algorithm=None), text_signature = "(name, algorithm=None)")]
    fn load_or_create(
        py: Python,
        name: String,
        algorithm: Option<SignerAlgorithms>,
    ) -> PyResult<Py<Signer>> {
        if let Some((existing, _)) = maybe_load_signer(Some(&name), true)? {
            return Py::new(py, existing);
        }

        let signer = create_local_signer(algorithm)?;
        let signer = save_signer(&signer, Some(&name))?;
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
        if let Some((existing, signer_type)) = maybe_load_signer(name.as_deref(), _load_if_exists)?
        {
            if let SignerType::VCompNotarySigner(vcomp_signer) = signer_type {
                // save blobs/statements incase the were purged previously
                persist_vcomp_signer_data(py, vcomp_signer)?;
            }
            return Py::new(py, existing);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let signer = rt.block_on(VCompNotarySigner::create(&url, None))?;
        persist_vcomp_signer_data(py, signer.clone())?;

        let signer_type = SignerType::VCompNotarySigner(signer);
        log::debug!(
            "Saving VCOMP Signer with did key '{}'",
            signer_type.get_did_doc().id
        );
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

        if let Some((existing, _)) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
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

    /// Imports a signer from a base64-encoded private key and persists it.
    ///
    /// Use this when you already have key material that should be reused by the
    /// SDK instead of generating a fresh signer with `Signer.new(...)`. The
    /// `algorithm` must match the provided private key bytes.
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
        if let Some((existing, _)) = maybe_load_signer(name.as_deref(), _load_if_exists)? {
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

/// Sets the active signer from a signer instance.
///
/// # Arguments
/// * `signer` - Signer instance
#[pyfunction]
#[pyo3(signature = (signer), text_signature = "(signer: Signer) -> None")]
fn set_active_signer(py: Python, signer: &Bound<'_, Signer>) -> PyResult<()> {
    let name = signer.borrow().name.clone();
    with_cfg!(py, |cfg| {
        log::debug!("Setting '{name}' as the active");
        let signer_file = cfg.app_dir.join(SIGNER_DIR).join(&name);
        if !signer_file.exists() {
            return Err(anyhow::anyhow!("No Signer named '{name}' found"));
        }

        let signer = utils_load_signer(signer_file)?;
        Config::set_active_signer_async(signer, Some(name.clone())).await?;
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

fn signer_exists(name: Option<&str>) -> PyResult<()> {
    if name.is_none() {
        return Ok(());
    }

    let signer_dir = cfg_blocking()?.app_dir.join(SIGNER_DIR);
    let name = name.unwrap();
    log::debug!("Adding Signer. Args= {name}");

    if fs::exists(signer_dir.join(name)).expect("Error checking if signer exists") {
        // This is where a script that ran once already lands on its second run, so
        // point at the call that makes it idempotent rather than just saying no.
        let msg = format!(
            "A signer named {name:?} already exists. Use \
             Signer.load_or_create(name={name:?}) to reuse it across runs, or \
             Signer.load({name:?}) to load it."
        );
        log::warn!("{msg}");
        return Err(PyErr::new::<PyValueError, _>(msg));
    }

    Ok(())
}

// Generates a new local signing key. Does not persist it.
fn create_local_signer(algorithm: Option<SignerAlgorithms>) -> PyResult<SignerType> {
    let signer = match algorithm.unwrap_or(SignerAlgorithms::ED25519).into() {
        KeyType::SECP256K1 => {
            log::debug!("Generating a new secp256k1 signer");
            SignerType::SECP256K1(Secp256k1Signer::create()?)
        }
        KeyType::SECP256R1 => {
            log::debug!("Generating a new secp256r1 signer");
            SignerType::P256(P256Signer::create()?)
        }
        KeyType::ED25519 => {
            log::debug!("Generating a new ed25519 signer");
            SignerType::ED25519(Ed25519Signer::create()?)
        }
    };
    Ok(signer)
}

// `_load_if_exists` predates `Signer.load` / `Signer.load_or_create`. It shipped in
// published examples and docs despite the underscore, so it stays functional until a
// breaking release.
fn warn_load_if_exists_deprecated(py: Python, replacement: &str) -> PyResult<()> {
    let message = CString::new(format!(
        "_load_if_exists is deprecated and will be removed in a future release; \
         use {replacement} instead"
    ))
    .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

    // stacklevel=2 points the warning at the caller, not at this shim.
    PyErr::warn(py, &py.get_type::<PyDeprecationWarning>(), &message, 2)
}

// Attempts to load signer data from disk, if `load_if_exists`== true && `name` is Some()
fn maybe_load_signer(
    name: Option<&str>,
    load_if_exists: bool,
) -> PyResult<Option<(Signer, SignerType)>> {
    if !load_if_exists {
        signer_exists(name)?;
        return Ok(None);
    }

    let Some(name) = name else {
        return Err(PyErr::new::<PyValueError, _>(
            "_load_if_exists=True requires a signer name",
        ));
    };

    log::info!("Attempting to load exiting signer {name:?}");

    let signer_path = cfg_blocking()?.app_dir.join(SIGNER_DIR).join(name);
    if !signer_path.exists() {
        return Ok(None);
    }

    let signer = utils_load_signer(signer_path)?;
    let did_key = signer.get_did_doc().id;
    Ok(Some((
        Signer {
            name: name.to_owned(),
            did_key,
        },
        signer,
    )))
}

// Save vcomp blobs to blob store and statements to sql
fn persist_vcomp_signer_data(py: Python, vcomp_signer: VCompNotarySigner) -> PyResult<()> {
    with_cfg!(py, |cfg| {
        if let Some(blobs) = vcomp_signer.did_blobs {
            log::debug!("Saving {} vcomp blobs to store", blobs.len());
            for (cid, data) in blobs {
                let codec = get_multicodec(&cid)?;
                cfg.blob_store.put(data, codec, Some(&cid)).await?;
            }
        }
        if let Some(statements) = vcomp_signer.credentials {
            log::debug!("Saving {} vcomp statements to store", statements.len());
            for (_, statement) in statements {
                let id = cfg.default_context.id;
                let s = serde_json::from_value::<Statement>(statement)?;
                cfg.sql_lite.register_statement(&s, &id).await?;
            }
        }
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
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
