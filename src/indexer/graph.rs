use serde::{Deserialize, Serialize};
use uuid::Uuid;

use integrity::lineage::models::statements::Statement;

/// A graph structure for organizing related statements hierarchically.
///
/// Graphs group statements together with optional parent-child relationships,
/// enabling versioning and organizational structure for lineage data.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Graph {
    /// Unique identifier for this graph
    #[sqlx(rename = "graph_id")]
    pub id: Uuid,
    /// Human-readable name for this graph
    pub name: String,
    /// Optional parent graph ID for hierarchical organization
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(rename = "parent_id")]
    pub parent: Option<Uuid>,
    /// Statements contained in this graph (populated on retrieval)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(skip)]
    pub statements: Option<Vec<Statement>>,
}
