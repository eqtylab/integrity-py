use std::{collections::HashSet, path::PathBuf};

use integrity::cid::{
    blake3::blake3_cid_raw_binary,
    iroh::{compute_dir_cid, compute_file_cid},
};
use pyo3::{
    exceptions::{PyAttributeError, PyRuntimeError, PyTypeError},
    prelude::*,
    types::{PyAny, PyDict, PyIterator},
    Bound, IntoPyObjectExt,
};

use crate::{indexer::Graph, metadata::Metadata, statements, with_ctx};

#[pyclass(subclass)]
pub struct Asset {
    #[pyo3(get)]
    pub statement_ids: Vec<String>,
    ctx: Option<Graph>,
    value: Py<PyAny>,
    cid: String,
    is_dir: bool,
    asset_type: String,
    skip_proof: bool,
    metadata: Py<PyAny>,
}

#[pymethods]
impl Asset {
    #[new]
    #[pyo3(signature = (obj, asset_type, cid, is_dir, ctx=None, **kwargs))]
    fn new(
        py: Python,
        obj: Py<PyAny>,
        asset_type: Py<PyAny>,
        cid: String,
        is_dir: bool,
        ctx: Option<Graph>,
        kwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let asset_type = resolve_asset_type(asset_type.bind(py))?;
        let kwargs_bound = kwargs.as_ref().map(|k| k.bind(py));
        let kwargs_dict = kwargs_bound
            .as_ref()
            .map(|k| k.cast::<PyDict>())
            .transpose()?;
        let (metadata, skip_proof, skip_registration) =
            build_metadata(py, asset_type.as_str(), &cid, kwargs_dict)?;

        let mut asset = Asset {
            ctx,
            value: obj,
            cid,
            is_dir,
            asset_type,
            skip_proof,
            metadata,
            statement_ids: Vec::new(),
        };

        if !skip_registration {
            asset.create_eqty_statements(py)?;
        }

        Ok(asset)
    }

    #[staticmethod]
    #[pyo3(signature = (obj, asset_type, ctx=None, store=None, **kwargs))]
    fn _from_object(
        py: Python,
        obj: Py<PyAny>,
        asset_type: Py<PyAny>,
        ctx: Option<Graph>,
        store: Option<bool>,
        kwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let serialized = serialize_for_hashing(py, obj.clone_ref(py))?;
        let cid = compute_cid_for_bytes(py, &serialized, store)?;
        Asset::new(py, obj, asset_type, cid, false, ctx, kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (path, asset_type, ctx=None, store=None, **kwargs))]
    fn _from_path(
        py: Python,
        path: PathBuf,
        asset_type: Py<PyAny>,
        ctx: Option<Graph>,
        store: Option<bool>,
        kwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let (cid, is_dir) = compute_cid_for_path(py, &path, store)?;
        let obj = path.into_py_any(py)?;
        Asset::new(py, obj, asset_type, cid, is_dir, ctx, kwargs)
    }

    #[staticmethod]
    #[pyo3(signature = (cid, asset_type, ctx=None, **kwargs))]
    fn _from_cid(
        py: Python,
        cid: String,
        asset_type: Py<PyAny>,
        ctx: Option<Graph>,
        kwargs: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let obj = cid.clone().into_py_any(py)?;
        Asset::new(py, obj, asset_type, cid, false, ctx, kwargs)
    }

    #[staticmethod]
    fn _factory_with_context(ctx: Graph, asset_type: Py<PyAny>) -> PyResult<Py<PyAny>> {
        Python::attach(|py| {
            let module = py.import("eqty_sdk.asset.asset")?;
            let factory = module.getattr("_Factory")?;
            factory.call1((ctx, asset_type))?.into_py_any(py)
        })
    }

    #[getter]
    fn name(&self, py: Python) -> PyResult<String> {
        let name = self.metadata.bind(py).getattr("name")?;
        if name.is_none() {
            Ok(String::new())
        } else {
            name.extract::<String>()
        }
    }

    #[getter]
    fn cid(&self) -> String {
        self.cid.clone()
    }

    #[getter]
    fn asset_type(&self) -> String {
        self.asset_type.clone()
    }

    #[getter]
    fn value(&self, py: Python) -> Py<PyAny> {
        self.value.clone_ref(py)
    }

    fn add_declaration<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        declaration: Py<PyAny>,
    ) -> PyResult<Py<Asset>> {
        let document_cid: String = declaration.bind(py).call_method0("cid")?.extract()?;
        let graph_id = slf.ctx.as_ref().map(|graph| graph.id);
        let ids = statements::governance::add_governance_statement(
            py,
            slf.cid.clone(),
            document_cid,
            Some(slf.skip_proof),
            graph_id,
        )?;
        slf.statement_ids.extend(ids);
        Ok(slf.into())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Asset({:?})", self.cid))
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        self.value.bind(py).str()?.extract()
    }

    fn __getattr__(&self, py: Python, key: &str) -> PyResult<Py<PyAny>> {
        match key {
            "asset_type" => return self.asset_type.clone().into_py_any(py),
            "cid" => return self.cid.clone().into_py_any(py),
            "name" => return self.name(py)?.into_py_any(py),
            "value" => return Ok(self.value.clone_ref(py)),
            _ => {}
        }

        if let Ok(attr) = self.value.bind(py).getattr(key) {
            return attr.into_py_any(py);
        }

        let metadata = self.metadata.bind(py);
        if metadata
            .call_method1("__contains__", (key,))?
            .extract::<bool>()?
        {
            let val = metadata.call_method1("__getitem__", (key,))?;
            return val.into_py_any(py);
        }

        Err(PyAttributeError::new_err(format!(
            "'{}' object has no attribute '{}'",
            self.value.bind(py).get_type().name()?,
            key
        )))
    }

    fn __setattr__(&mut self, py: Python, key: &str, value: Py<PyAny>) -> PyResult<()> {
        let internal: HashSet<&str> = [
            "ctx",
            "value",
            "cid",
            "is_dir",
            "metadata",
            "asset_type",
            "skip_proof",
            "statement_ids",
        ]
        .into_iter()
        .collect();

        if internal.contains(key) {
            match key {
                "cid" => self.cid = value.extract(py)?,
                "asset_type" => self.asset_type = value.extract(py)?,
                _ => {}
            }
            return Ok(());
        }

        self.value.bind(py).setattr(key, value)?;
        Ok(())
    }

    fn __getitem__(&self, py: Python, key: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let item = self.value.bind(py).get_item(key)?;
        item.into_py_any(py)
    }

    fn __setitem__(&self, py: Python, key: Py<PyAny>, value: Py<PyAny>) -> PyResult<()> {
        self.value.bind(py).set_item(key, value)?;
        Ok(())
    }

    fn __iter__(&self, py: Python) -> PyResult<Py<PyAny>> {
        let iter = PyIterator::from_object(self.value.bind(py))?;
        iter.into_py_any(py)
    }

    fn __len__(&self, py: Python) -> PyResult<usize> {
        self.value.bind(py).len()
    }

    fn __hash__(&self, py: Python) -> PyResult<isize> {
        self.value.bind(py).hash()
    }

    fn __eq__(&self, py: Python, other: Py<PyAny>) -> PyResult<bool> {
        if let Ok(other_asset) = other.extract::<PyRef<Asset>>(py) {
            if self.value.bind(py).is_none() && other_asset.value.bind(py).is_none() {
                return Ok(true);
            }
            return self.value.bind(py).eq(other_asset.value.bind(py));
        }
        self.value.bind(py).eq(other)
    }

    fn __add__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__add__", (rhs,))?;
        out.into_py_any(py)
    }

    fn __mul__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__mul__", (rhs,))?;
        out.into_py_any(py)
    }

    fn __truediv__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__truediv__", (rhs,))?;
        out.into_py_any(py)
    }

    fn __floordiv__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__floordiv__", (rhs,))?;
        out.into_py_any(py)
    }

    fn __mod__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__mod__", (rhs,))?;
        out.into_py_any(py)
    }

    fn __pow__(
        &self,
        py: Python,
        other: Py<PyAny>,
        modulo: Option<Py<PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = if let Some(modulo) = modulo {
            self.value.bind(py).call_method1("__pow__", (rhs, modulo))?
        } else {
            self.value.bind(py).call_method1("__pow__", (rhs,))?
        };
        out.into_py_any(py)
    }

    fn __sub__(&self, py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let rhs = extract_other_value(py, other)?;
        let out = self.value.bind(py).call_method1("__sub__", (rhs,))?;
        out.into_py_any(py)
    }
}

impl Asset {
    fn create_eqty_statements(&mut self, py: Python) -> PyResult<()> {
        let graph_id = self.ctx.as_ref().map(|g| g.id);
        let mut data_ids = statements::data::add_data_statement(
            py,
            vec![self.cid.clone()],
            Some(self.skip_proof),
            graph_id,
        )?;
        self.statement_ids.append(&mut data_ids);

        let metadata_json: String = self
            .metadata
            .bind(py)
            .call_method0("to_json_str")?
            .extract()?;
        let mut metadata_ids = statements::metadata::add_metadata_statement(
            py,
            self.cid.clone(),
            metadata_json,
            Some(self.skip_proof),
            graph_id,
        )?;
        self.statement_ids.append(&mut metadata_ids);

        crate::maybe_create_model_signing_statement(
            py,
            self.cid.clone(),
            self.name(py)
                .unwrap_or_else(|_| "Unnamed Asset".to_string()),
            self.is_dir,
        )?;

        Ok(())
    }
}

fn extract_other_value(py: Python, other: Py<PyAny>) -> PyResult<Py<PyAny>> {
    if let Ok(other_asset) = other.extract::<PyRef<Asset>>(py) {
        Ok(other_asset.value.clone_ref(py))
    } else {
        Ok(other)
    }
}

fn resolve_asset_type(asset_type: &Bound<PyAny>) -> PyResult<String> {
    if let Ok(s) = asset_type.extract::<String>() {
        return Ok(s);
    }
    if let Ok(value) = asset_type.getattr("value") {
        return value.extract::<String>();
    }
    Err(PyErr::new::<PyTypeError, _>(
        "asset_type must be a string or Enum with a 'value' attribute",
    ))
}

fn build_metadata(
    py: Python,
    asset_type: &str,
    cid: &str,
    kwargs: Option<&Bound<PyDict>>,
) -> PyResult<(Py<PyAny>, bool, bool)> {
    let mut skip_proof = None;
    let mut skip_registration = false;

    let metadata_kwargs = PyDict::new(py);
    if let Some(kwargs) = kwargs {
        for (k, v) in kwargs.iter() {
            let key: String = k.extract()?;
            if key == "skip_proof" {
                skip_proof = Some(v.extract::<bool>()?);
                continue;
            }
            if key == "skip_registration" {
                skip_registration = v.extract::<bool>()?;
                continue;
            }
            metadata_kwargs.set_item(k, v)?;
        }
    }

    if !metadata_kwargs.contains("name")? {
        let name = format!("{asset_type}-{}", &cid[cid.len().saturating_sub(4)..]);
        metadata_kwargs.set_item("name", name)?;
    }
    metadata_kwargs.set_item("assetType", asset_type)?;

    let metadata = Py::new(py, Metadata::from_dict(metadata_kwargs))?;

    let skip_proof = skip_proof.unwrap_or_else(|| {
        std::env::var("EQTY_SKIP_PROOF")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    });

    Ok((metadata.into_py_any(py)?, skip_proof, skip_registration))
}

fn serialize_for_hashing(py: Python, obj: Py<PyAny>) -> PyResult<Vec<u8>> {
    let module = py.import("eqty_sdk.asset.asset")?;
    let func = module.getattr("serialize_for_hashing")?;
    let bytes = func.call1((obj,))?;
    bytes.extract::<Vec<u8>>()
}

fn compute_cid_for_bytes(py: Python, data: &[u8], store: Option<bool>) -> PyResult<String> {
    with_ctx!(py, |ctx| {
        let cid = blake3_cid_raw_binary(data)?;
        let store_flag = store.unwrap_or(ctx.store_all_blobs);

        if store_flag {
            let blob_dir = ctx.app_dir.join("blobs");
            tokio::fs::create_dir_all(&blob_dir).await?;
            let file_path = blob_dir.join(&cid);
            tokio::fs::write(&file_path, data).await?;
        }

        Ok(cid)
    })
}

fn compute_cid_for_path(
    py: Python,
    path: &PathBuf,
    store: Option<bool>,
) -> PyResult<(String, bool)> {
    with_ctx!(py, |ctx| {
        let store_flag = store.unwrap_or(ctx.store_all_blobs);
        let blob_dir = ctx.app_dir.join("blobs");
        tokio::fs::create_dir_all(&blob_dir).await?;

        if path.is_file() {
            let file_cid_result = compute_file_cid(path.clone(), ctx.hashing.clone()).await?;
            let cid = file_cid_result.cid.clone();

            if store_flag {
                let storage_path = blob_dir.join(&cid);
                tokio::fs::copy(&path, &storage_path).await?;
            }

            Ok((cid, false))
        } else if path.is_dir() {
            let dir_cid_result =
                compute_dir_cid(path.clone(), ctx.hashing.clone(), ctx.cid_ignore.clone()).await?;
            let cid = dir_cid_result.collection.cid.clone();

            tokio::fs::write(
                blob_dir.join(&dir_cid_result.collection.cid),
                dir_cid_result.collection.blob,
            )
            .await?;
            tokio::fs::write(
                blob_dir.join(&dir_cid_result.meta.cid),
                dir_cid_result.meta.blob,
            )
            .await?;

            if store_flag {
                for (file_name, file_cid) in dir_cid_result.file_hashes {
                    let src = path.join(file_name);
                    let dst = blob_dir.join(file_cid);
                    tokio::fs::copy(src, dst).await?;
                }
            }

            Ok((cid, true))
        } else {
            Err(PyRuntimeError::new_err(format!(
                "The provided path {path:?} was not found"
            )))
        }
    })
}
