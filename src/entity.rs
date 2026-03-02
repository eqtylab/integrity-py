//! Entity type for representing unhashed entities with UUIDs.

use anyhow::anyhow;
use integrity::lineage::models::statements::{
    EntityStatement, MetadataStatement, Statement, StatementTrait,
};
use pyo3::{prelude::*, types::PyList};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_ctx, Graph, CID,
};

/// Represents an unhashed entity with a UUID identifier.
///
/// Entities are used to represent objects that don't have a content-based
/// identifier (CID) but need a unique identifier for tracking purposes.
#[derive(Clone, Debug)]
#[pyclass]
pub struct Entity {
    #[pyo3(get)]
    uuid: String,
}

#[pymethods]
impl Entity {
    /// Create a new Entity with the given UUID string.
    #[new]
    fn new(uuid: String) -> Self {
        Entity { uuid }
    }

    /// Create a new Entity with a randomly generated UUID.
    #[staticmethod]
    fn generate() -> Self {
        Entity {
            uuid: Uuid::new_v4().to_string(),
        }
    }

    /// Create an Entity from a UUID string.
    #[staticmethod]
    fn from_uuid(uuid: String) -> Self {
        Entity { uuid }
    }

    fn __str__(&self) -> String {
        self.uuid.clone()
    }

    fn __repr__(&self) -> String {
        format!("Entity(\"{}\")", self.uuid)
    }
}

/// Creates a new Entity with a random UUID, registers entity and metadata statements.
///
/// # Arguments
/// * `metadata_json` - JSON string containing metadata to associate with the entity
/// * `skip_proof` - If true, skip creating a VC statement
/// * `timestamp` - Optional timestamp for statements
/// * `graph_id` - Optional graph ID to register statements to
///
/// # Returns
/// Tuple of (Entity, list of statement IDs)
#[pyfunction]
#[pyo3(signature = (metadata_json, skip_proof=None, timestamp=None, graph=None))]
pub fn create_entity(
    py: Python,
    metadata_json: String,
    skip_proof: Option<bool>,
    timestamp: Option<String>,
    graph: Option<Graph>,
) -> PyResult<(Entity, Py<PyList>)> {
    let entity = Entity::generate();
    let statement_ids = create_entity_statements(
        py,
        &entity.uuid,
        metadata_json,
        skip_proof,
        timestamp,
        graph,
    )?;
    Ok((entity, statement_ids))
}

/// Creates an Entity from an existing UUID, registers entity and metadata statements.
///
/// # Arguments
/// * `uuid` - UUID string for the entity
/// * `metadata_json` - JSON string containing metadata to associate with the entity
/// * `skip_proof` - If true, skip creating a VC statement
/// * `timestamp` - Optional timestamp for statements
/// * `graph` - Optional graph ID to register statements to
///
/// # Returns
/// Tuple of (Entity, list of statement IDs)
#[pyfunction]
#[pyo3(signature = (uuid, metadata_json, skip_proof=None, timestamp=None, graph=None))]
pub fn create_entity_from_uuid(
    py: Python,
    uuid: String,
    metadata_json: String,
    skip_proof: Option<bool>,
    timestamp: Option<String>,
    graph: Option<Graph>,
) -> PyResult<(Entity, Py<PyList>)> {
    let entity = Entity::from_uuid(uuid);
    let statement_ids = create_entity_statements(
        py,
        &entity.uuid,
        metadata_json,
        skip_proof,
        timestamp,
        graph,
    )?;
    Ok((entity, statement_ids))
}

/// Internal helper to create all statements for an entity
fn create_entity_statements(
    py: Python,
    entity_uuid: &str,
    metadata_json: String,
    skip_proof: Option<bool>,
    timestamp: Option<String>,
    graph: Option<Graph>,
) -> PyResult<Py<PyList>> {
    let mut statement_ids: Vec<CID> = Vec::new();
    let timestamp = resolve_timestamp(timestamp);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let entity_statement = Statement::EntityRegistration(
            EntityStatement::create(
                vec![entity_uuid.to_string()],
                registered_by.clone(),
                timestamp.clone(),
            )
            .await?,
        );
        let entity_statement_id: CID = entity_statement.get_id().into();
        ctx.sql_lite
            .register_statement(&entity_statement, &graph_id)
            .await?;
        statement_ids.push(entity_statement_id.clone());

        if !skip_proof {
            let vc_id =
                create_vc_for_statement(&ctx, &entity_statement_id, graph_id, timestamp.clone())
                    .await?;
            statement_ids.push(vc_id);
        };

        let metadata_value: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| anyhow!("Invalid metadata JSON: {e}"))?;

        let signer = ctx
            .active_signer
            .clone()
            .ok_or_else(|| anyhow!("No active signer available"))?;

        let metadata_statement = MetadataStatement::create_from_json(
            entity_statement_id.to_string(),
            metadata_value,
            signer.get_did_doc().id,
            timestamp,
        )
        .await?;

        let metadata_stmt = Statement::MetadataRegistration(metadata_statement);
        let metadata_id = metadata_stmt.get_id();
        ctx.sql_lite
            .register_statement(&metadata_stmt, &graph_id)
            .await?;
        statement_ids.push(metadata_id.into());

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(PyList::new(py, statement_ids)?.unbind())
}

/// `entity` submodule.
#[pymodule]
pub fn entity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Entity>()?;
    m.add_function(wrap_pyfunction!(create_entity, m)?)?;
    m.add_function(wrap_pyfunction!(create_entity_from_uuid, m)?)?;
    Ok(())
}
