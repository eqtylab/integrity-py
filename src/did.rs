use anyhow::Context as AnyhowContext;
use integrity::{
    blob_store::BlobStore,
    lineage::models::statements::{Statement, StatementTrait},
    signer::{load_signer as utils_load_signer, SignerType},
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict, Bound};

use crate::{
    config::cfg_blocking,
    signer::{Signer, SIGNER_DIR},
    statements, with_cfg, CID,
};

/// DID object
#[derive(Clone)]
#[pyclass(from_py_object)]
pub struct DID {
    /// DID string used for registration.
    #[pyo3(get)]
    pub did: String,
}

#[pymethods]
impl DID {
    #[new]
    #[pyo3(signature = (did, **kwargs))]
    fn new(py: Python, did: String, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        build_did(py, did, None, kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (signer, **kwargs))]
    fn from_signer(
        py: Python,
        signer: Py<Signer>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let did = signer.bind(py).borrow().did_key.clone();
        build_did(py, did, Some(signer), kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (did, **kwargs))]
    fn from_did_string(
        py: Python,
        did: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        build_did(py, did, None, kwargs)
    }
}

fn build_did(
    py: Python,
    did: String,
    signer: Option<Py<Signer>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<DID> {
    log::debug!("Building signer. DID: {did}");
    let metadata_json = if let Some(kwargs) = kwargs {
        let json = py.import("json")?;
        json.getattr("dumps")?
            .call1((kwargs,))?
            .extract::<String>()?
    } else {
        "{}".to_string()
    };

    let mut statement_ids: Vec<CID> = Vec::new();

    let loaded_signer = signer
        .as_ref()
        .map(|signer| signer.bind(py).borrow().name.clone())
        .map(|name| load_saved_signer(&name))
        .transpose()?;

    if let Some(SignerType::VCompNotarySigner(vcomp_signer)) = loaded_signer {
        let mut vcomp_statement_ids = with_cfg!(py, |cfg| {
            let mut ids = Vec::new();

            if let Some(statements) = vcomp_signer.did_statements {
                for value in statements.values() {
                    let statement: Statement =
                        serde_json::from_value(value.clone()).context("Invalid statement")?;
                    let id = CID::new(statement.get_id());
                    cfg.sql_lite
                        .register_statement(&statement, &cfg.default_context.id)
                        .await?;
                    ids.push(id);
                }
            }

            if let Some(blobs) = vcomp_signer.did_blobs {
                for (_, data) in blobs {
                    cfg.blob_store.put(data, 0, None).await?;
                }
            }

            Ok::<_, anyhow::Error>(ids)
        })?;

        statement_ids.append(&mut vcomp_statement_ids);
    } else {
        let mut did_ids = statements::did::add_did_statement(py, did.clone(), None, None)?;
        statement_ids.append(&mut did_ids);
    }

    let mut metadata_ids =
        statements::metadata::add_metadata_statement(py, did.clone(), metadata_json, None, None)?;
    statement_ids.append(&mut metadata_ids);

    Ok(DID { did })
}

fn load_saved_signer(name: &str) -> PyResult<SignerType> {
    let cfg = cfg_blocking()?;
    let signer_file = cfg.app_dir.join(SIGNER_DIR).join(name);
    if !signer_file.exists() {
        return Err(PyValueError::new_err(format!(
            "No Signer named '{name}' found. Save or register the signer before using it."
        )));
    }
    Ok(utils_load_signer(signer_file)?)
}
