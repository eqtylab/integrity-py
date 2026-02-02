use serde_json::Value;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

/// Database row representing an association between statements.
///
/// Links statements through subject-association relationships,
/// enabling graph-based queries and traversal.
#[derive(Debug, sqlx::FromRow)]
pub struct AssociationRow {
    /// Unique identifier for this association
    pub id: String,
    /// The subject statement ID
    pub subject: String,
    /// The associated statement ID
    pub association: String,
}

/// Database row representing a statement with optional metadata and credentials.
///
/// Contains the core statement data along with optional metadata,
/// verifiable credentials, and DID documents.
#[derive(Debug)]
pub struct StatementRow {
    /// The statement content as JSON
    pub statement: Value,
    /// Optional metadata associated with the statement
    pub metadata: Option<Value>,
    /// Optional verifiable credential for the statement
    pub vc: Option<Value>,
    /// Optional DID document for the statement
    pub did: Option<Value>,
}

impl<'r> FromRow<'r, SqliteRow> for StatementRow {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        fn parse_json(s: String) -> Result<Value, sqlx::Error> {
            serde_json::from_str(&s).map_err(|e| sqlx::Error::Decode(Box::new(e)))
        }

        fn parse_optional_json(s: Option<String>) -> Result<Option<Value>, sqlx::Error> {
            s.map(parse_json).transpose()
        }

        Ok(StatementRow {
            statement: parse_json(row.try_get("statement")?)?,
            metadata: parse_optional_json(row.try_get("metadata")?)?,
            vc: parse_optional_json(row.try_get("vc")?)?,
            did: parse_optional_json(row.try_get("did")?)?,
        })
    }
}
