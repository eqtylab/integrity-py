use anyhow::{anyhow, Context as AnyhowContext, Result};
use integrity::{
    blob_store::BlobStore,
    lineage::models::statements::{Statement, StatementTrait},
    signer::{load_signer as utils_load_signer, SignerType},
};
use pyo3::{prelude::*, types::PyDict, Bound};
use uuid::uuid;

use crate::{
    config::cfg_blocking,
    signer::{Signer, SIGNER_DIR},
    statements, with_cfg, CID,
};

/// DID object
#[pyclass]
pub struct DID {
    /// DID string used for registration.
    #[pyo3(get)]
    pub did: String,
    // /// IDs of statements created for this DID.
    // #[pyo3(get)]
    // pub statement_ids: Vec<CID>,
}

#[pymethods]
impl DID {
    #[new]
    #[pyo3(signature = (did, signer=None, **kwargs))]
    fn new(
        py: Python,
        did: String,
        signer: Option<Py<Signer>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        build_did(py, did, signer, kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (signer, **kwargs))]
    fn from_signer(
        py: Python,
        signer: Py<Signer>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let did_key = signer.bind(py).borrow().did_key.clone();
        build_did(py, did_key, Some(signer), kwargs)
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

    let is_vcomp_signer = if let Some(signer) = signer.as_ref() {
        let signer_name = signer.bind(py).borrow().name.clone();
        is_vcomp_signer(&signer_name)?
    } else {
        false
    };

    if is_vcomp_signer {
        let signer_name = signer
            .as_ref()
            .map(|s| s.bind(py).borrow().name.clone())
            .unwrap_or_default();

        let mut vcomp_statement_ids = with_cfg!(py, |cfg| {
            let signer_file = cfg.app_dir.join(SIGNER_DIR).join(&signer_name);
            if !signer_file.exists() {
                return Err(anyhow!("No Signer named '{signer_name}' found"));
            }

            let signer = utils_load_signer(signer_file)?;
            let mut ids = Vec::new();

            if let SignerType::VCompNotarySigner(vcomp_signer) = signer {
                if let Some(statements) = vcomp_signer.did_statements {
                    for value in statements.values() {
                        let statement: Statement =
                            serde_json::from_value(value.clone()).context("Invalid statement")?;
                        let id = CID::new(statement.get_id());
                        let dummy_ctx = uuid!("00000000-0000-0000-0000-000000000000");
                        cfg.sql_lite
                            .register_statement(&statement, &dummy_ctx)
                            .await?;
                        ids.push(id);
                    }
                }

                if let Some(blobs) = vcomp_signer.did_blobs {
                    for (_, data) in blobs {
                        cfg.blob_store.put(data, 0, None).await?;
                    }
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

fn is_vcomp_signer(name: &str) -> Result<bool> {
    log::trace!("Checking if {name} is a known vcomp signer");
    let cfg = cfg_blocking()?;
    let signer_file = cfg.app_dir.join(SIGNER_DIR).join(name);
    if !signer_file.exists() {
        log::trace!("{name} is not vcomp");
        return Ok(false);
    }
    let signer = utils_load_signer(signer_file)?;
    Ok(matches!(signer, SignerType::VCompNotarySigner(_)))
}
