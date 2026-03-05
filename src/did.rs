use anyhow::{anyhow, Context as AnyhowContext, Result};
use integrity::{
    blob_store::BlobStore,
    lineage::models::statements::{Statement, StatementTrait},
    signer::{load_signer as utils_load_signer, SignerType},
};
use pyo3::{prelude::*, types::PyDict, Bound};

use crate::{
    config::cfg_blocking,
    indexer::Context,
    signer::{Signer, SIGNER_DIR},
    statements, with_cfg, CID,
};

/// A DID statement result bound to a context.
#[pyclass]
pub struct DID {
    /// Context where the DID statement was registered.
    #[pyo3(get)]
    pub ctx: Context,
    /// DID string used for registration.
    #[pyo3(get)]
    pub did: String,
    /// IDs of statements created for this DID.
    #[pyo3(get)]
    pub statement_ids: Vec<CID>,
}

/// Builder for DID statements in a specific context.
#[pyclass]
pub struct DidFactory {
    ctx: Context,
}

#[pymethods]
impl DID {
    #[new]
    #[pyo3(signature = (ctx, did, signer=None, **kwargs))]
    fn new(
        py: Python,
        ctx: Context,
        did: String,
        signer: Option<Py<Signer>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        build_did(py, ctx, did, signer, kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (signer, **kwargs))]
    fn from_signer(
        py: Python,
        signer: Py<Signer>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let default_context = cfg_blocking()?.default_context.clone();
        let did_key = signer.bind(py).borrow().did_key.clone();
        build_did(py, default_context, did_key, Some(signer), kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (did, **kwargs))]
    fn from_did_string(
        py: Python,
        did: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let default_context = cfg_blocking()?.default_context.clone();
        build_did(py, default_context, did, None, kwargs)
    }

    #[staticmethod]
    fn with_context(ctx: Context) -> DidFactory {
        DidFactory { ctx }
    }
}

#[pymethods]
impl DidFactory {
    #[pyo3(signature = (signer, **kwargs))]
    fn build_from_signer(
        &self,
        py: Python,
        signer: Py<Signer>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<DID> {
        let did_key = signer.bind(py).borrow().did_key.clone();
        build_did(py, self.ctx.clone(), did_key, Some(signer), kwargs)
    }

    #[pyo3(signature = (did, **kwargs))]
    fn build_from_did_string(
        &self,
        py: Python,
        did: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<DID> {
        build_did(py, self.ctx.clone(), did, None, kwargs)
    }
}

fn build_did(
    py: Python,
    ctx: Context,
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
                        cfg.sql_lite.register_statement(&statement, &ctx.id).await?;
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
        let mut did_ids =
            statements::did::add_did_statement(py, did.clone(), None, Some(ctx.clone()))?;
        statement_ids.append(&mut did_ids);
    }

    let mut metadata_ids = statements::metadata::add_metadata_statement(
        py,
        did.clone(),
        metadata_json,
        None,
        Some(ctx.clone()),
    )?;
    statement_ids.append(&mut metadata_ids);

    Ok(DID {
        ctx,
        did,
        statement_ids,
    })
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
