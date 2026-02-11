use anyhow::{anyhow, Context as AnyhowContext};
use integrity::{
    lineage::models::statements::{Statement, StatementTrait},
    signer::{load_signer as utils_load_signer, SignerType},
};
use pyo3::{prelude::*, types::PyDict, Bound};

use crate::{config::ctx_blocking, indexer::Graph, signer::Signer, statements, with_ctx};

#[pyclass]
pub struct Did {
    #[pyo3(get)]
    pub ctx: Graph,
    #[pyo3(get)]
    pub statement_ids: Vec<String>,
}

#[pyclass]
pub struct DidFactory {
    ctx: Graph,
}

#[pymethods]
impl Did {
    #[new]
    #[pyo3(signature = (ctx, did, signer=None, **kwargs))]
    fn new(
        py: Python,
        ctx: Graph,
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
        let default_graph = ctx_blocking()?.default_graph.clone();
        let did_key = signer.bind(py).borrow().did_key.clone();
        build_did(py, default_graph, did_key, Some(signer), kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (did, **kwargs))]
    fn from_did_string(
        py: Python,
        did: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let default_graph = ctx_blocking()?.default_graph.clone();
        build_did(py, default_graph, did, None, kwargs)
    }

    #[staticmethod]
    fn with_context(ctx: Graph) -> DidFactory {
        DidFactory { ctx }
    }
}

#[pymethods]
impl DidFactory {
    #[pyo3(signature = (signer, **kwargs))]
    fn from_signer(
        &self,
        py: Python,
        signer: Py<Signer>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Did> {
        let did_key = signer.bind(py).borrow().did_key.clone();
        build_did(py, self.ctx.clone(), did_key, Some(signer), kwargs)
    }

    #[pyo3(signature = (did, **kwargs))]
    fn from_did_string(
        &self,
        py: Python,
        did: String,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Did> {
        build_did(py, self.ctx.clone(), did, None, kwargs)
    }
}

fn build_did(
    py: Python,
    ctx: Graph,
    did: String,
    signer: Option<Py<Signer>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Did> {
    let metadata_json = if let Some(kwargs) = kwargs {
        let json = py.import("json")?;
        json.getattr("dumps")?
            .call1((kwargs,))?
            .extract::<String>()?
    } else {
        "{}".to_string()
    };

    let mut statement_ids: Vec<String> = Vec::new();

    let is_vcomp_signer = if let Some(signer) = signer.as_ref() {
        let signer_name = signer.bind(py).borrow().name.clone();
        is_vcomp_signer(&signer_name, py)?
    } else {
        false
    };

    if is_vcomp_signer {
        let signer_name = signer
            .as_ref()
            .map(|s| s.bind(py).borrow().name.clone())
            .unwrap_or_default();

        let mut vcomp_statement_ids = with_ctx!(py, |cfg| {
            let signer_file = cfg.app_dir.join("signers").join(&signer_name);
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
                        let id = statement.get_id();
                        cfg.sql_lite.register_statement(&statement, &ctx.id).await?;
                        ids.push(id);
                    }
                }

                if let Some(blobs) = vcomp_signer.did_blobs {
                    let blob_dir = cfg.app_dir.join("blobs");
                    tokio::fs::create_dir_all(&blob_dir).await?;
                    for (cid, data) in blobs {
                        let blob_path = blob_dir.join(cid);
                        tokio::fs::write(blob_path, data).await?;
                    }
                }
            }

            Ok::<_, anyhow::Error>(ids)
        })?;

        statement_ids.append(&mut vcomp_statement_ids);
    } else {
        let mut did_ids = statements::did::add_did_statement(py, did.clone(), None, Some(ctx.id))?;
        statement_ids.append(&mut did_ids);
    }

    let mut metadata_ids =
        statements::metadata::add_metadata_statement(py, did, metadata_json, None, Some(ctx.id))?;
    statement_ids.append(&mut metadata_ids);

    Ok(Did { ctx, statement_ids })
}

fn is_vcomp_signer(name: &str, py: Python) -> PyResult<bool> {
    let result = with_ctx!(py, |cfg| {
        let signer_file = cfg.app_dir.join("signers").join(name);
        if !signer_file.exists() {
            return Err(anyhow!("No Signer named '{name}' found"));
        }
        let signer = utils_load_signer(signer_file)?;
        Ok::<_, anyhow::Error>(matches!(signer, SignerType::VCompNotarySigner(_)))
    })?;
    Ok(result)
}
