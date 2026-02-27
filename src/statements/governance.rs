use integrity::lineage::models::statements::{GovernanceStatement, Statement, StatementTrait};
use pyo3::{pyfunction, PyResult, Python};

use crate::{
    config::create_vc_for_statement, resolve_skip_proof, resolve_timestamp, with_ctx, Graph,
};

#[pyfunction]
#[pyo3(signature = (subject, document, *, skip_proof=None, graph=None))]
pub fn add_governance_statement(
    py: Python,
    subject: String,
    document: String,
    skip_proof: Option<bool>,
    graph: Option<Graph>,
) -> PyResult<Vec<String>> {
    let timestamp = resolve_timestamp(None);
    let skip_proof = resolve_skip_proof(skip_proof);

    with_ctx!(py, |ctx| {
        let graph_id = ctx.resolve_graph_id(graph);
        let registered_by = ctx.clone().get_active_signer_did_key()?;

        let statement = Statement::GovernanceRegistration(
            GovernanceStatement::create(subject, document, registered_by, timestamp.clone())
                .await?,
        );

        ctx.sql_lite
            .register_statement(&statement, &graph_id)
            .await?;

        let id = statement.get_id();
        let mut statement_ids = vec![id.clone()];

        if !skip_proof {
            let vc_id = create_vc_for_statement(&ctx, &id, graph_id, timestamp).await?;
            statement_ids.push(vc_id);
        }

        Ok(statement_ids)
    })
}
