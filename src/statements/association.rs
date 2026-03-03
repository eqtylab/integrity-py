use integrity::lineage::models::statements::{
    AssociationStatement, AssociationType, Statement, StatementTrait,
};
use pyo3::prelude::*;

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_ctx, Graph, CID,
};

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyAssociationType {
    Certifies,
    Includes,
    IsInstanceOf,
}

impl From<PyAssociationType> for AssociationType {
    fn from(value: PyAssociationType) -> Self {
        match value {
            PyAssociationType::Certifies => AssociationType::Certifies,
            PyAssociationType::Includes => AssociationType::Includes,
            PyAssociationType::IsInstanceOf => AssociationType::IsInstanceOf,
        }
    }
}

#[pyfunction]
#[pyo3(signature = (subject, association, association_type, *, skip_proof=None, graph=None))]
pub fn add_association_statement(
    py: Python,
    subject: String,
    association: Vec<String>,
    association_type: PyAssociationType,
    skip_proof: Option<bool>,
    graph: Option<Graph>,
) -> PyResult<Vec<CID>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let registered_by = ctx.clone().get_active_signer_did_key()?;
        let ass_type = AssociationType::from(association_type);

        let statement = Statement::AssociationRegistration(
            AssociationStatement::create(
                subject,
                association,
                ass_type,
                registered_by,
                timestamp.clone(),
            )
            .await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id: CID = statement.get_id().into();
        let mut statement_ids: Vec<CID> = vec![id.clone()];

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        };

        Ok(statement_ids)
    })
}
