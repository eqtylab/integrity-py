use std::collections::{HashMap, HashSet};

use anyhow::Result;
use sqlx::{sqlite::SqliteRow, SqlitePool};
use uuid::Uuid;

use super::{rows_to_statements, AssociationRow, Graph};
use integrity::lineage::models::statements::{Statement, StatementTrait};

/// Provides persistent storage for statements organized in graphs
/// with support for hierarchical relationships and queries.
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    fn parse_statement_rows(rows: Vec<SqliteRow>) -> Result<HashMap<String, Statement>> {
        rows_to_statements(rows)
    }

    /// Initializes the database schema by creating all necessary tables and indexes.
    pub async fn init(&self) -> Result<()> {
        let comp_tables = r#"
            CREATE TABLE IF NOT EXISTS computation_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            );
        "#;
        sqlx::query(comp_tables).execute(&self.pool).await?;

        let data_tables = r#"
            CREATE TABLE IF NOT EXISTS data_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS data_statement_subjects (
                statement_id TEXT NOT NULL,
                subject TEXT NOT NULL,
                PRIMARY KEY (statement_id, subject),
                FOREIGN KEY (statement_id) REFERENCES data_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_data_statement_subjects_subject ON data_statement_subjects(subject);
        "#;
        sqlx::query(data_tables).execute(&self.pool).await?;

        let metadata_table = r#"
            CREATE TABLE IF NOT EXISTS metadata_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                subject TEXT NOT NULL
            );
        "#;
        sqlx::query(metadata_table).execute(&self.pool).await?;

        let storage_table = r#"
            CREATE TABLE IF NOT EXISTS storage_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                data TEXT NOT NULL
            );
        "#;
        sqlx::query(storage_table).execute(&self.pool).await?;

        let entity_table = r#"
            CREATE TABLE IF NOT EXISTS entity_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entity_statement_subjects (
                statement_id TEXT NOT NULL,
                entity TEXT NOT NULL,
                PRIMARY KEY (statement_id, entity),
                FOREIGN KEY (statement_id) REFERENCES entity_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_entity_statement_subjects_subject ON entity_statement_subjects(entity);
        "#;
        sqlx::query(entity_table).execute(&self.pool).await?;

        let association_table = r#"
            CREATE TABLE IF NOT EXISTS association_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                subject TEXT NOT NULL,
                association TEXT NOT NULL
            );
        "#;
        sqlx::query(association_table).execute(&self.pool).await?;

        let graph_tables = r#"
            CREATE TABLE IF NOT EXISTS statement_graph_link (
                statement_id TEXT NOT NULL,
                graph_id TEXT NOT NULL,
                PRIMARY KEY (statement_id, graph_id),
                FOREIGN KEY (graph_id) REFERENCES graphs(graph_id)
            );

            CREATE TABLE IF NOT EXISTS graphs (
                graph_id TEXT PRIMARY KEY,
                name TEXT,
                parent_id TEXT,
                FOREIGN KEY (parent_id) REFERENCES graphs(graph_id)
            );
        "#;
        sqlx::query(graph_tables).execute(&self.pool).await?;

        let sigstore_table = r#"
            CREATE TABLE IF NOT EXISTS sigstore_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            );
        "#;
        sqlx::query(sigstore_table).execute(&self.pool).await?;

        let credential_table = r#"
            CREATE TABLE IF NOT EXISTS credential_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                credential_subject TEXT NOT NULL
            );
        "#;
        sqlx::query(credential_table).execute(&self.pool).await?;

        let dsse_table = r#"
            CREATE TABLE IF NOT EXISTS dsse_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            );
        "#;
        sqlx::query(dsse_table).execute(&self.pool).await?;

        let did_table = r#"
            CREATE TABLE IF NOT EXISTS did_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                type TEXT NOT NULL,
                did TEXT NOT NULL
            );
        "#;
        sqlx::query(did_table).execute(&self.pool).await?;

        let governance_table = r#"
            CREATE TABLE IF NOT EXISTS governance_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                subject TEXT NOT NULL,
                document TEXT NOT NULL
            );
        "#;
        sqlx::query(governance_table).execute(&self.pool).await?;

        Ok(())
    }

    /// Creates a new SQLite indexer connected to the database at the given file path.
    ///
    /// # Arguments
    /// * `database` - SQLite database connection string (e.g., "sqlite://path/to/db.sqlite")
    pub async fn new(database: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database).await?;

        Ok(Self { pool })
    }

    /// Helper to handle optionally associating a statement with a graph_id
    async fn opt_associate_statement_to_graph(
        &self,
        statement_id: &str,
        graph_id: Option<&Uuid>,
    ) -> Result<()> {
        if let Some(graph_id) = graph_id {
            log::debug!("Registering '{statement_id}' under graph {graph_id:?}");
            self.associate_statement_to_graph(statement_id, graph_id)
                .await?;
        }
        Ok(())
    }

    /// Used to register statements associtated with a specific graph_id (aka NON-Global)
    async fn register_graph_statement(
        &self,
        statement: &Statement,
        graph_id: Option<&Uuid>,
    ) -> Result<()> {
        match statement {
            Statement::ComputationRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering computation '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO computation_statements
                    (id, statement, registered_by) VALUES (?1, ?2, ?3)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await
            }
            Statement::DataRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering data '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO data_statements
                    (id, statement, registered_by) VALUES (?1, ?2, ?3)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await?;

                for data_item in s.data.to_vec_string() {
                    sqlx::query(
                        r#"
                      INSERT OR IGNORE INTO data_statement_subjects
                      (statement_id, subject) VALUES (?1, ?2)
                    "#,
                    )
                    .bind(&id)
                    .bind(data_item)
                    .execute(&self.pool)
                    .await?;
                }
                Ok(())
            }
            Statement::MetadataRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering metadata '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO metadata_statements
                    (id, statement, registered_by, subject) VALUES (?1, ?2, ?3, ?4)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.subject)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await
            }
            Statement::StorageRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering storage '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO storage_statements
                    (id, statement, registered_by, data) VALUES (?1, ?2, ?3, ?4)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.data)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await
            }
            Statement::EntityRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering entity '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO entity_statements
                    (id, statement, registered_by) VALUES (?1, ?2, ?3)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await?;

                for entity in s.entity.to_vec_string() {
                    sqlx::query(
                        r#"
                      INSERT OR IGNORE INTO entity_statement_subjects
                      (statement_id, entity) VALUES (?1, ?2)
                    "#,
                    )
                    .bind(&id)
                    .bind(entity)
                    .execute(&self.pool)
                    .await?;
                }

                Ok(())
            }
            Statement::AssociationRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering association '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO association_statements
                    (id, statement, registered_by, subject, association) VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.subject)
                .bind(&s.association)
                .execute(&self.pool)
                .await?;

                self.opt_associate_statement_to_graph(&id, graph_id).await
            }
            Statement::CredentialSigstoreBundleRegistration(_)
            | Statement::DidRegistration(_)
            | Statement::GovernanceRegistration(_)
            | Statement::CredentialDsseRegistration(_)
            | Statement::CredentialRegistration(_) => {
                log::error!(
                    "Attempted to register a non-graph specific statement '{}' to a graph",
                    statement.get_type_string().unwrap_or("UNKNOWN".to_owned())
                );
                Ok(())
            }
        }
    }

    async fn register_global_statement(&self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::CredentialSigstoreBundleRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering sigstore bundle '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO sigstore_statements
                    (id, statement, registered_by) VALUES (?1, ?2, ?3)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .execute(&self.pool)
                .await?;

                Ok(())
            }
            Statement::CredentialRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering credential '{id}'");
                let subject = s
                    .credential
                    .credential_subject
                    .first()
                    .and_then(|s| s.id.as_ref())
                    .map(|id| id.to_string())
                    .unwrap_or_default();

                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO credential_statements
                    (id, statement, registered_by, credential_subject) VALUES (?1, ?2, ?3, ?4)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&subject)
                .execute(&self.pool)
                .await?;

                Ok(())
            }
            Statement::CredentialDsseRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering dsse '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO dsse_statements
                    (id, statement, registered_by) VALUES (?1, ?2, ?3)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .execute(&self.pool)
                .await?;

                Ok(())
            }
            Statement::DidRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let registered_by = s.get_registered_by();
                let type_ = s.get_type();
                let did = s.get_did();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering {type_} did '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO did_statements
                    (id, statement, registered_by, type, did) VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(registered_by)
                .bind(type_)
                .bind(did)
                .execute(&self.pool)
                .await?;

                Ok(())
            }
            Statement::GovernanceRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering governance '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO governance_statements
                    (id, statement, registered_by, subject, document) VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.subject)
                .bind(&s.document)
                .execute(&self.pool)
                .await?;

                Ok(())
            }
            Statement::ComputationRegistration(_)
            | Statement::AssociationRegistration(_)
            | Statement::DataRegistration(_)
            | Statement::MetadataRegistration(_)
            | Statement::StorageRegistration(_)
            | Statement::EntityRegistration(_) => {
                log::error!(
                    "Attempted to register a graph specific statement '{}' to the global store",
                    statement.get_type_string().unwrap_or("UNKNOWN".to_owned())
                );
                Ok(())
            }
        }
    }

    async fn get_global_statements(
        &self,
        statements: &mut HashMap<String, Statement>,
    ) -> Result<()> {
        // TODO: Get the Credential, CredDsse, CredSigStore, DID, Governance Statements
        // for ALL the previously fetched statements regardless of project
        let mut dids = HashSet::new();
        let mut credential_subjects = HashSet::new();

        for stmt in statements.values() {
            dids.insert(stmt.get_registered_by().to_owned());
            credential_subjects.insert(stmt.get_id().to_owned());
        }

        log::debug!("Getting credential statements for subjects: {credential_subjects:?}");
        let placeholders = vec!["?"; credential_subjects.len()].join(", ");
        let global_query = format!(
            r#"
            SELECT statement
            FROM credential_statements
            WHERE credential_subject IN ({})
        "#,
            placeholders
        );

        let mut sql_query = sqlx::query(&global_query);
        for credential_subject in &credential_subjects {
            sql_query = sql_query.bind(credential_subject);
        }

        let vc_rows = sql_query.fetch_all(&self.pool).await?;
        log::debug!("Found '{}' credential statements", vc_rows.len());

        let vc_statements = Self::parse_statement_rows(vc_rows)?;
        statements.extend(vc_statements);

        if !dids.is_empty() {
            log::debug!("Getting DID statements for subjects: {dids:?}");
            let placeholders = vec!["?"; dids.len()].join(", ");
            let global_query = format!(
                r#"
                SELECT
                  did.statement as statement
                  ,meta.statement as metadata
                  ,vc.statement as vc
                FROM did_statements did
                LEFT JOIN metadata_statements meta ON did.did = meta.subject
                LEFT JOIN credential_statements vc ON did.id = vc.credential_subject
                WHERE did IN ({})"#,
                placeholders
            );

            let mut sql_query = sqlx::query(&global_query);
            for did in &dids {
                sql_query = sql_query.bind(did);
            }

            let did_rows = sql_query.fetch_all(&self.pool).await?;
            log::debug!("Found '{}' did statements", did_rows.len());

            let did_statements = Self::parse_statement_rows(did_rows)?;
            statements.extend(did_statements);
        }

        Ok(())
    }

    /// Creates a record in the "graphs" table
    pub async fn create_graph(
        &self,
        graph_id: &Uuid,
        name: &str,
        parent_id: Option<&Uuid>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO graphs
            (graph_id, name, parent_id)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(graph_id.to_string())
        .bind(name)
        .bind(parent_id.map(|id| id.to_string()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Registers a statement in the database, optionally associating it with a graph.
    ///
    /// Graph-specific statements (computation, data, metadata, etc.) are linked to the
    /// provided graph_id. Global statements (credentials, DIDs, governance) are stored
    /// without graph association.
    pub async fn register_statement(
        &self,
        statement: &Statement,
        graph_id: Option<&Uuid>,
    ) -> Result<()> {
        log::trace!("Registering statement");
        match statement {
            Statement::ComputationRegistration(_)
            | Statement::AssociationRegistration(_)
            | Statement::DataRegistration(_)
            | Statement::MetadataRegistration(_)
            | Statement::StorageRegistration(_)
            | Statement::EntityRegistration(_) => {
                self.register_graph_statement(statement, graph_id).await
            }
            Statement::CredentialSigstoreBundleRegistration(_)
            | Statement::CredentialRegistration(_)
            | Statement::DidRegistration(_)
            | Statement::GovernanceRegistration(_)
            | Statement::CredentialDsseRegistration(_) => {
                self.register_global_statement(statement).await
            }
        }
    }

    /// Updates the link table to assign a statement to a graph
    pub async fn associate_statement_to_graph(
        &self,
        statement_id: &str,
        graph_id: &Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO statement_graph_link
            (statement_id, graph_id)
            VALUES (?1, ?2)
        "#,
        )
        .bind(statement_id)
        .bind(graph_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieves a graph and all its associated statements by graph ID.
    ///
    /// Returns the graph with its statements populated, including statements
    /// from parent graphs in the hierarchy.
    pub async fn retrieve_graph(&self, graph_id: &Uuid) -> Result<Graph> {
        log::info!("Retrieving statements for graph {graph_id:?}");

        let mut graph: Graph = sqlx::query_as(
            r#"
            SELECT graph_id, name, parent_id
            FROM graphs g
            WHERE g.graph_id = ?1
        "#,
        )
        .bind(graph_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        // Create placeholders for the IN clause
        let compute_query_str = r#"
            SELECT
              s.statement
              , vc.statement as vc
              , metadata.statement as metadata
              , did.statement as did
            FROM computation_statements s
            LEFT JOIN statement_graph_link l ON s.id = l.statement_id
            LEFT JOIN graphs g ON l.graph_id = g.graph_id
            LEFT JOIN credential_statements vc ON vc.credential_subject = s.id
            LEFT JOIN metadata_statements metadata on metadata.subject = s.id
            LEFT JOIN did_statements did ON s.registered_by = did.did
            WHERE g.graph_id = ?1
        "#;

        let compute_rows = sqlx::query(compute_query_str)
            .bind(graph_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        if compute_rows.is_empty() {
            log::info!("No computation statements found for graph(s) {graph_id:?}");
            return Ok(graph);
        }

        let mut subjects: Vec<String> = Vec::new();

        log::debug!("Found '{}' compute statements", compute_rows.len());
        let mut statements = Self::parse_statement_rows(compute_rows)?;
        for statement in statements.values() {
            if let Statement::ComputationRegistration(s) = statement {
                subjects.extend(s.input.to_vec_string());
                subjects.extend(s.output.to_vec_string());
            }
        }

        // Get the Data & Metadata & Storage & Association & Entity statements
        // WHERE MD.subject/data/association IN compute.[inputs + outputs] AND in project or parent project
        log::debug!("Getting statements for subjects: {subjects:?}");
        // Build placeholders: (?2, ?3, ?4)
        let placeholders: Vec<String> = (2..=subjects.len() + 1)
            .map(|i| format!("?{}", i))
            .collect();
        let in_clause = format!("({})", placeholders.join(", "));

        // Gets all the statements registered under <graph_id> and it's parents
        let query = format!(
            r#"
            WITH RECURSIVE graph_hierarchy AS (
                SELECT graph_id, name, parent_id, 0 as level
                FROM graphs
                WHERE graph_id = ?1
                UNION ALL
                SELECT g.graph_id, g.name, g.parent_id, gh.level + 1
                FROM graphs g
                JOIN graph_hierarchy gh ON g.graph_id = gh.parent_id
            )
            SELECT DISTINCT
                COALESCE(data.statement, metadata.statement, storage.statement, association.statement) as statement,
                NULL as metadata,
                NULL as vc,
                NULL as did,
                gh.graph_id,
                gh.level
            FROM graph_hierarchy gh
            LEFT JOIN statement_graph_link sgl ON gh.graph_id = sgl.graph_id
            LEFT JOIN data_statements data ON sgl.statement_id = data.id
            LEFT JOIN data_statement_subjects dss ON data.id = dss.statement_id
            LEFT JOIN metadata_statements metadata ON sgl.statement_id = metadata.id
            LEFT JOIN storage_statements storage ON sgl.statement_id = storage.id
            LEFT JOIN association_statements association ON sgl.statement_id = association.id
            LEFT JOIN entity_statements entity ON sgl.statement_id = entity.id
            LEFT JOIN entity_statement_subjects ess ON entity.id = ess.statement_id
            WHERE COALESCE(dss.subject, metadata.subject, storage.data, association.association, association.subject, ess.entity) IN {}
            ORDER BY gh.level;
            "#,
            in_clause
        );

        let mut sql_query = sqlx::query(&query).bind(graph_id.to_string());

        for subject in &subjects {
            sql_query = sql_query.bind(subject);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;
        log::debug!("Found '{}' related statements", rows.len());
        statements.extend(Self::parse_statement_rows(rows)?);

        self.get_global_statements(&mut statements).await?;

        graph.statements = Some(statements.into_values().collect());
        Ok(graph)
    }

    /// Returns all association IDs linked to the given subject.
    pub async fn get_associations_for_subject(&self, subject: &str) -> Result<Vec<String>> {
        log::trace!("Retrieving associations for subject={subject}.");

        let rows: Vec<AssociationRow> = sqlx::query_as(
            r#"
            SELECT id, subject, association
            FROM association_statements
            WHERE subject = $1
            "#,
        )
        .bind(subject)
        .fetch_all(&self.pool)
        .await?;

        let mut associations = rows.into_iter().map(|r| r.association).collect::<Vec<_>>();

        associations.sort();
        associations.dedup();

        Ok(associations)
    }

    /// Returns all subject IDs linked to the given association.
    pub async fn get_subjects_for_association(&self, association: &str) -> Result<Vec<String>> {
        log::trace!("Retrieving subjects for association={association}.");

        let rows: Vec<AssociationRow> = sqlx::query_as(
            r#"
            SELECT id, subject, association
            FROM association_statements
            WHERE association = $1
            "#,
        )
        .bind(association)
        .fetch_all(&self.pool)
        .await?;

        let mut subjects = rows.into_iter().map(|r| r.subject).collect::<Vec<_>>();

        subjects.sort();
        subjects.dedup();

        Ok(subjects)
    }

    /// Returns metadata for all graphs in the database.
    pub async fn get_graph_info(&self) -> Result<Vec<Graph>> {
        log::trace!("Retrieving all graph info");

        let rows: Vec<Graph> = sqlx::query_as(
            r#"
            SELECT graph_id, name, parent_id
            FROM graphs
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Returns metadata for all descendant graphs of the given parent.
    pub async fn get_child_graph_info(&self, parent_id: &Uuid) -> Result<Vec<Graph>> {
        log::trace!("Retrieving child graph info for {parent_id:?}");

        let rows: Vec<Graph> = sqlx::query_as(
            r#"
            WITH RECURSIVE descendants AS (
                -- Base case: direct children
                SELECT graph_id, name, parent_id
                FROM graphs
                WHERE parent_id = ?1

                UNION ALL

                -- Recursive case: children of children
                SELECT g.graph_id, g.name, g.parent_id
                FROM graphs g
                INNER JOIN descendants d ON g.parent_id = d.graph_id
            )
            SELECT graph_id, name, parent_id
            FROM descendants
            "#,
        )
        .bind(parent_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::Sqlite;
    use integrity::lineage::models::statements::{
        AssociationStatement, ComputationStatement, DataStatement, MetadataStatement, Statement,
        StatementTrait, StorageStatement,
    };

    async fn setup_db() -> Sqlite {
        let db = Sqlite::new("sqlite::memory:").await.unwrap();
        db.init().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_graph() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000001");
        db.create_graph(&graph_id, "test:1", None).await.unwrap();

        // Verify by retrieving the graph
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        assert_eq!(graph.id, graph_id);
        assert_eq!(graph.name, "test:1");
        assert!(graph.parent.is_none());
    }

    #[tokio::test]
    async fn test_create_graph_with_parent() {
        let db = setup_db().await;
        let parent_id = uuid::uuid!("00000000-0000-0000-0000-1000000000F0");
        let parent_name = "test:parent";
        db.create_graph(&parent_id, parent_name, None)
            .await
            .unwrap();

        let parent_graph = db.retrieve_graph(&parent_id).await.unwrap();
        assert_eq!(parent_graph.id, parent_id);
        assert_eq!(parent_graph.name, parent_name);
        assert!(parent_graph.parent.is_none());

        let child_id = uuid::uuid!("00000000-0000-0000-0000-1000000000F1");
        let child_name = "test:parent:child";
        db.create_graph(&child_id, child_name, Some(&parent_id))
            .await
            .unwrap();

        let child_graph = db.retrieve_graph(&child_id).await.unwrap();
        assert_eq!(child_graph.id, child_id);
        assert_eq!(child_graph.name, child_name);
        assert_eq!(child_graph.parent, Some(parent_id));
    }

    #[tokio::test]
    async fn test_register_computation_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000010");
        db.create_graph(&graph_id, "comp_test", None).await.unwrap();

        let did = String::from("did:key:comp_statement");
        let statement = ComputationStatement::create(
            None,
            vec![String::from("urn:cid:input1")],
            vec![String::from("urn:cid:output1")],
            did.clone(),
            None,
            did.clone(),
            None,
        )
        .await
        .unwrap();
        let statement_id = statement.get_id();

        let comp_statement = Statement::ComputationRegistration(statement);
        db.register_statement(&comp_statement, Some(&graph_id))
            .await
            .unwrap();

        // Verify by retrieving the graph
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        let statements = graph.statements.as_ref().unwrap();
        assert!(statements.iter().any(|s| s.get_id() == statement_id));
    }

    #[tokio::test]
    async fn test_register_data_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000011");
        db.create_graph(&graph_id, "data_test", None).await.unwrap();

        let did = String::from("did:key:data_statement");
        let data_cid = String::from("urn:cid:input1");
        let statement = DataStatement::create(vec![data_cid.clone()], did.clone(), None)
            .await
            .unwrap();
        let data_statement_id = statement.get_id();

        let data_statement = Statement::DataRegistration(statement);
        db.register_statement(&data_statement, Some(&graph_id))
            .await
            .unwrap();

        // Create a computation that references this data so it gets retrieved
        let comp_statement = ComputationStatement::create(
            None,
            vec![data_cid],
            vec![String::from("urn:cid:output1")],
            did.clone(),
            None,
            did,
            None,
        )
        .await
        .unwrap();
        db.register_statement(
            &Statement::ComputationRegistration(comp_statement),
            Some(&graph_id),
        )
        .await
        .unwrap();

        // Verify by retrieving the graph - should contain both computation and data statements
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        let statements = graph.statements.as_ref().unwrap();
        assert!(statements.iter().any(|s| s.get_id() == data_statement_id));
    }

    #[tokio::test]
    async fn test_register_metadata_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000012");
        db.create_graph(&graph_id, "metadata_test", None)
            .await
            .unwrap();

        let did = String::from("did:key:metadata_statement");
        let subject = String::from("urn:cid:metadata1");
        let metadata = String::from("the metadata");
        let statement =
            MetadataStatement::create(subject.clone(), metadata.clone(), did.clone(), None)
                .await
                .unwrap();
        let metadata_statement_id = statement.get_id();

        let meta_statement = Statement::MetadataRegistration(statement);
        db.register_statement(&meta_statement, Some(&graph_id))
            .await
            .unwrap();

        // Create a computation that references this subject so it gets retrieved
        let comp_statement = ComputationStatement::create(
            None,
            vec![subject],
            vec![String::from("urn:cid:output1")],
            did.clone(),
            None,
            did,
            None,
        )
        .await
        .unwrap();
        db.register_statement(
            &Statement::ComputationRegistration(comp_statement),
            Some(&graph_id),
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        let statements = graph.statements.as_ref().unwrap();
        assert!(statements
            .iter()
            .any(|s| s.get_id() == metadata_statement_id));
    }

    #[tokio::test]
    async fn test_register_storage_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000013");
        db.create_graph(&graph_id, "storage_test", None)
            .await
            .unwrap();

        let did = String::from("did:key:storage_statement");
        let subject = String::from("urn:cid:storage");
        let stored_on = String::from("urn:cid:stored_on");
        let statement =
            StorageStatement::create(subject.clone(), stored_on.clone(), None, did.clone(), None)
                .await
                .unwrap();
        let storage_statement_id = statement.get_id();

        let storage_statement = Statement::StorageRegistration(statement);
        db.register_statement(&storage_statement, Some(&graph_id))
            .await
            .unwrap();

        // Create a computation that references the subject (data field) so it gets retrieved
        let comp_statement = ComputationStatement::create(
            None,
            vec![subject], // Use subject, not stored_on
            vec![String::from("urn:cid:output1")],
            did.clone(),
            None,
            did,
            None,
        )
        .await
        .unwrap();
        db.register_statement(
            &Statement::ComputationRegistration(comp_statement),
            Some(&graph_id),
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        let statements = graph.statements.as_ref().unwrap();
        assert!(statements
            .iter()
            .any(|s| s.get_id() == storage_statement_id));
    }

    #[tokio::test]
    async fn test_register_association_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000014");
        db.create_graph(&graph_id, "association_test", None)
            .await
            .unwrap();

        let did = String::from("did:key:association_statement");
        let subject = String::from("urn:cid:association_subjectx");
        let association = String::from("urn:cid:association_associate");
        let statement =
            AssociationStatement::create(subject.clone(), association.clone(), did.clone(), None)
                .await
                .unwrap();
        let association_statement_id = statement.get_id();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, Some(&graph_id))
            .await
            .unwrap();

        // Create a computation that references the association so it gets retrieved
        let comp_statement = ComputationStatement::create(
            None,
            vec![association],
            vec![String::from("urn:cid:output1")],
            did.clone(),
            None,
            did,
            None,
        )
        .await
        .unwrap();
        db.register_statement(
            &Statement::ComputationRegistration(comp_statement),
            Some(&graph_id),
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let graph = db.retrieve_graph(&graph_id).await.unwrap();
        let statements = graph.statements.as_ref().unwrap();
        assert!(statements
            .iter()
            .any(|s| s.get_id() == association_statement_id));
    }

    #[tokio::test]
    async fn test_association_get_by_subject() {
        let db = setup_db().await;
        let did = String::from("did:key:association_statement");
        let subject = String::from("urn:cid:association_subject");
        let associate1 = String::from("urn:cid:association_first");
        let statement =
            AssociationStatement::create(subject.clone(), associate1.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, None).await.unwrap();

        let associations = db.get_associations_for_subject(&subject).await.unwrap();
        assert_eq!(associations.len(), 1);
        assert_eq!(associations.first(), Some(&associate1));

        let associate2 = String::from("urn:cid:association_second");
        let statement =
            AssociationStatement::create(subject.clone(), associate2.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, None).await.unwrap();
        let associations = db.get_associations_for_subject(&subject).await.unwrap();
        assert_eq!(associations.len(), 2);
        assert_eq!(associations[0], associate1);
        assert_eq!(associations[1], associate2);
    }

    #[tokio::test]
    async fn test_association_get_by_association() {
        let db = setup_db().await;
        let did = String::from("did:key:association_statement");
        let subject1 = String::from("urn:cid:association_subject1");
        let associate = String::from("urn:cid:association");
        let statement =
            AssociationStatement::create(subject1.clone(), associate.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, None).await.unwrap();

        let subjects = db.get_subjects_for_association(&associate).await.unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects.first(), Some(&subject1));

        let subject2 = String::from("urn:cid:association_subject2");
        let statement =
            AssociationStatement::create(subject2.clone(), associate.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, None).await.unwrap();
        let subjects = db.get_subjects_for_association(&associate).await.unwrap();
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[0], subject1);
        assert_eq!(subjects[1], subject2);
    }

    #[tokio::test]
    async fn test_statement_retrieval_with_hierarchy() {
        let db = setup_db().await;
        let root_graph_id = uuid::uuid!("00000000-0000-0000-0000-500000000001");
        db.create_graph(&root_graph_id, "Root Graph", None)
            .await
            .unwrap();

        let child_graph_id = uuid::uuid!("00000000-0000-0000-0000-500000000002");
        db.create_graph(&child_graph_id, "Child Graph", Some(&root_graph_id))
            .await
            .unwrap();

        let child_graph_id_2 = uuid::uuid!("00000000-0000-0000-0000-500000000003");
        db.create_graph(&child_graph_id_2, "Child Graph 2", Some(&child_graph_id))
            .await
            .unwrap();

        let input_data = vec![
            "urn:cid:comp_data_input_1".to_owned(),
            "urn:cid:comp_data_input_2".to_owned(),
        ];
        let statement =
            DataStatement::create(input_data.clone(), "did:key:unit_test".to_owned(), None)
                .await
                .unwrap();

        let data_input = Statement::DataRegistration(statement);
        db.register_statement(&data_input, Some(&root_graph_id))
            .await
            .unwrap();

        let output_data = vec!["urn:cid:comp_data_output".to_owned()];
        let statement =
            DataStatement::create(output_data.clone(), "did:key:unit_test".to_owned(), None)
                .await
                .unwrap();

        let data_output = Statement::DataRegistration(statement);
        db.register_statement(&data_output, Some(&child_graph_id))
            .await
            .unwrap();

        let statement = MetadataStatement::create(
            "urn:cid:comp_data_input_1".to_owned(),
            String::from("metadata"),
            "did:key:metadata".to_owned(),
            None,
        )
        .await
        .unwrap();

        let metadata = Statement::MetadataRegistration(statement);
        db.register_statement(&metadata, Some(&child_graph_id_2))
            .await
            .unwrap();

        let did = String::from("did:key:comp_statement");
        let statement = ComputationStatement::create(
            None,
            input_data,
            output_data,
            did.clone(),
            None,
            did,
            None,
        )
        .await
        .unwrap();

        let comp_statement = Statement::ComputationRegistration(statement);
        db.register_statement(&comp_statement, Some(&child_graph_id))
            .await
            .unwrap();

        // Check that the statements in the parent graphs get pulled in
        let graph = db.retrieve_graph(&child_graph_id).await.unwrap();
        assert_eq!(graph.statements.as_ref().unwrap().len(), 3);

        // Register the same statement in a lower child project
        db.register_statement(&comp_statement, Some(&child_graph_id_2))
            .await
            .unwrap();

        // Check that the statements in the parent graphs get pulled in from a lower child
        let graph = db.retrieve_graph(&child_graph_id_2).await.unwrap();
        assert_eq!(graph.statements.as_ref().unwrap().len(), 4);
    }
}
