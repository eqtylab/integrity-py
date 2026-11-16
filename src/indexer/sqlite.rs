use std::collections::{HashMap, HashSet};

use anyhow::{Context as _, Result};
use integrity::lineage::models::statements::{Statement, StatementTrait};
use sqlx::{Row, Sqlite as SqliteDb, SqlitePool, Transaction};
use uuid::Uuid;

use super::{rows_to_statements, Context};

/// Extracts the subject ID from a credential subject, returning an error if the ID is missing.
fn extract_credential_subject_id(subject: &serde_json::Value) -> Result<String> {
    subject
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Credential subject missing or malformed 'id' field"))
}

/// Statement tables whose rows belong to the graphs that link them, and are deleted once
/// no graph links them any more.
const GRAPH_OWNED_STATEMENT_TABLES: [&str; 7] = [
    "computation_statements",
    "data_statements",
    "metadata_statements",
    "storage_statements",
    "entity_statements",
    "association_statements",
    "governance_statements",
];

/// A recursive CTE named `graph_hierarchy`, holding the graph bound to `?1` and its ancestors.
const GRAPH_HIERARCHY_CTE: &str = r#"
    graph_hierarchy(graph_id) AS (
        SELECT graph_id FROM graphs WHERE graph_id = ?1
        UNION ALL
        SELECT g.parent_id FROM graphs g
        JOIN graph_hierarchy gh ON g.graph_id = gh.graph_id
        WHERE g.parent_id IS NOT NULL
    )
"#;

/// Whether credential `vc` belongs in an export of `graph_hierarchy`.
///
/// A credential about a DID or a DID statement is always included, because exports find
/// those by DID. Any other credential is included only if it is linked to the exported graph
/// or one of its ancestors, so an export leaves out other graphs' credentials for the same
/// statement.
const CREDENTIAL_IN_HIERARCHY: &str = r#"
    (
        vc.credential_subject LIKE 'did:%'
        OR vc.credential_subject IN (SELECT id FROM did_statements)
        OR EXISTS (
            SELECT 1 FROM statement_graph_link vc_link
            JOIN graph_hierarchy gh ON vc_link.graph_id = gh.graph_id
            WHERE vc_link.statement_id = vc.id
        )
    )
"#;

/// Provides persistent storage for statements organized in graphs
/// with support for hierarchical relationships and queries.
pub struct Sqlite {
    pool: SqlitePool,
}

#[cfg(test)]
impl Sqlite {
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl Sqlite {
    async fn retrieve_related_statements(
        &self,
        graph_id: &Uuid,
        subjects: &[String],
    ) -> Result<Vec<sqlx::sqlite::SqliteRow>> {
        if subjects.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<String> = (2..=subjects.len() + 1)
            .map(|i| format!("?{}", i))
            .collect();
        let in_clause = format!("({})", placeholders.join(", "));

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
                COALESCE(data.statement, metadata.statement, storage.statement, association.statement, entity.statement, governance.statement) as statement,
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
            LEFT JOIN association_statement_items asi ON association.id = asi.statement_id
            LEFT JOIN entity_statements entity ON sgl.statement_id = entity.id
            LEFT JOIN entity_statement_subjects ess ON entity.id = ess.statement_id
            LEFT JOIN governance_statements governance ON sgl.statement_id = governance.id
            WHERE dss.subject IN {}
               OR metadata.subject IN {}
               OR storage.data IN {}
               OR asi.association_item IN {}
               OR association.subject IN {}
               OR ess.entity IN {}
               OR governance.subject IN {}
            ORDER BY gh.level;
            "#,
            in_clause, in_clause, in_clause, in_clause, in_clause, in_clause, in_clause
        );

        let mut sql_query = sqlx::query(&query).bind(graph_id.to_string());
        for subject in subjects {
            sql_query = sql_query.bind(subject);
        }

        sql_query.fetch_all(&self.pool).await.map_err(Into::into)
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

            CREATE INDEX IF NOT EXISTS idx_metadata_statements_subject
                ON metadata_statements(subject);
        "#;
        sqlx::query(metadata_table).execute(&self.pool).await?;

        let storage_table = r#"
            CREATE TABLE IF NOT EXISTS storage_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                data TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_storage_statements_data
                ON storage_statements(data);
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
                type TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS association_statement_items (
                statement_id TEXT NOT NULL,
                association_item TEXT NOT NULL,
                PRIMARY KEY (statement_id, association_item),
                FOREIGN KEY (statement_id) REFERENCES association_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_association_statement_items_item
                ON association_statement_items(association_item);

            CREATE INDEX IF NOT EXISTS idx_association_statements_subject
                ON association_statements(subject);
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

            CREATE INDEX IF NOT EXISTS idx_statement_graph_link_graph_id
                ON statement_graph_link(graph_id);
        "#;
        sqlx::query(graph_tables).execute(&self.pool).await?;

        let sigstore_table = r#"
            CREATE TABLE IF NOT EXISTS sigstore_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                subject TEXT
            );
        "#;
        sqlx::query(sigstore_table).execute(&self.pool).await?;
        self.migrate_sigstore_statements_subject_column().await?;

        let credential_table = r#"
            CREATE TABLE IF NOT EXISTS credential_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL,
                credential_subject TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_credential_statements_subject
                ON credential_statements(credential_subject);
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

            CREATE INDEX IF NOT EXISTS idx_governance_statements_subject
                ON governance_statements(subject);
        "#;
        sqlx::query(governance_table).execute(&self.pool).await?;

        Ok(())
    }

    async fn migrate_sigstore_statements_subject_column(&self) -> Result<()> {
        let columns = sqlx::query("PRAGMA table_info(sigstore_statements)")
            .fetch_all(&self.pool)
            .await?;
        let has_subject_column = columns.iter().any(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == "subject")
                .unwrap_or(false)
        });

        if !has_subject_column {
            sqlx::query("ALTER TABLE sigstore_statements ADD COLUMN subject TEXT")
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sigstore_statements_subject
            ON sigstore_statements(subject)
            "#,
        )
        .execute(&self.pool)
        .await?;

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

    /// Begins a transaction that holds the write lock from the start.
    ///
    /// Use it for a transaction that reads before it writes. In a deferred transaction, SQLite
    /// fails that first write at once with "database is locked" if another connection is
    /// committing, without waiting for the busy timeout.
    async fn begin_write(&self) -> Result<Transaction<'static, SqliteDb>> {
        Ok(self.pool.begin_with("BEGIN IMMEDIATE").await?)
    }

    /// Creates a record in the "graphs" table
    pub async fn create_graph(&self, context: &Context) -> Result<()> {
        if let Some(parent_id) = context.parent {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO graphs
                (graph_id, name, parent_id)
                VALUES (?1, ?2, NULL)
                "#,
            )
            .bind(parent_id.to_string())
            .bind(parent_id.to_string())
            .execute(&self.pool)
            .await?;
        }
        sqlx::query(
            r#"
            INSERT INTO graphs
            (graph_id, name, parent_id)
            VALUES (?1, ?2, ?3)
            ON CONFLICT (graph_id) DO NOTHING
            "#,
        )
        .bind(context.id.to_string())
        .bind(context.name.clone())
        .bind(context.parent.map(|id| id.to_string()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Deletes a graph and all descendant graphs, along with statements and credentials
    /// that no remaining graph links.
    pub async fn delete_graph_tree(&self, graph_id: &Uuid) -> Result<()> {
        // Get all graph ids in the subtree (including the root).
        log::debug!("Deleting graph tree for {graph_id}");
        let mut transaction = self.begin_write().await?;
        let graph_rows = sqlx::query(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT graph_id
                FROM graphs
                WHERE graph_id = ?1
                UNION ALL
                SELECT g.graph_id
                FROM graphs g
                INNER JOIN descendants d ON g.parent_id = d.graph_id
            )
            SELECT graph_id FROM descendants
            "#,
        )
        .bind(graph_id.to_string())
        .fetch_all(&mut *transaction)
        .await?;

        if graph_rows.is_empty() {
            log::debug!("No graphs found for {graph_id}");
            return Ok(());
        }

        let graph_ids: Vec<String> = graph_rows
            .into_iter()
            .map(|row| row.get::<String, _>("graph_id"))
            .collect();
        log::debug!("Found {} graph(s) to delete", graph_ids.len());

        // Collect all statement ids linked to these graphs.
        let placeholders = vec!["?"; graph_ids.len()].join(", ");
        let stmt_query = format!(
            "SELECT DISTINCT statement_id FROM statement_graph_link WHERE graph_id IN ({})",
            placeholders
        );
        let mut stmt_sql = sqlx::query(&stmt_query);
        for gid in &graph_ids {
            stmt_sql = stmt_sql.bind(gid);
        }
        let stmt_rows = stmt_sql.fetch_all(&mut *transaction).await?;
        let statement_ids: Vec<String> = stmt_rows
            .into_iter()
            .map(|row| row.get::<String, _>("statement_id"))
            .collect();
        log::debug!("Found {} statement(s) to delete", statement_ids.len());

        // Delete links first.
        let link_query = format!(
            "DELETE FROM statement_graph_link WHERE graph_id IN ({})",
            placeholders
        );
        let mut link_sql = sqlx::query(&link_query);
        for gid in &graph_ids {
            link_sql = link_sql.bind(gid);
        }
        link_sql.execute(&mut *transaction).await?;
        log::debug!("Deleted statement graph links");

        Self::delete_unlinked_statements(&mut transaction, &statement_ids).await?;

        // Check foreign keys at commit, so the order graphs are deleted in doesn't matter.
        // This lasts until the transaction ends.
        sqlx::query("PRAGMA defer_foreign_keys = ON;")
            .execute(&mut *transaction)
            .await?;
        let graph_del_query = format!("DELETE FROM graphs WHERE graph_id IN ({})", placeholders);
        let mut graph_del_sql = sqlx::query(&graph_del_query);
        for gid in &graph_ids {
            graph_del_sql = graph_del_sql.bind(gid);
        }
        graph_del_sql.execute(&mut *transaction).await?;
        log::debug!("Deleted graphs");

        transaction.commit().await?;
        Ok(())
    }

    /// Deletes a graph and the statements and credentials that no remaining graph links,
    /// only if it has no child graphs.
    pub async fn delete_graph_no_children(&self, graph_id: &Uuid) -> Result<()> {
        log::debug!("Deleting graph without children: {graph_id}");
        let mut transaction = self.begin_write().await?;
        let child_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM graphs
            WHERE parent_id = ?1
            "#,
        )
        .bind(graph_id.to_string())
        .fetch_one(&mut *transaction)
        .await?;

        if child_count.0 > 0 {
            log::debug!("Graph {graph_id} has {} child graph(s)", child_count.0);
            return Err(anyhow::anyhow!(
                "Graph has child graphs; delete_tree required"
            ));
        }

        // Collect statement ids linked to this graph.
        let stmt_rows = sqlx::query(
            r#"
            SELECT DISTINCT statement_id
            FROM statement_graph_link
            WHERE graph_id = ?1
            "#,
        )
        .bind(graph_id.to_string())
        .fetch_all(&mut *transaction)
        .await?;

        let statement_ids: Vec<String> = stmt_rows
            .into_iter()
            .map(|row| row.get::<String, _>("statement_id"))
            .collect();
        log::debug!(
            "Found {} statement(s) linked to graph {graph_id}",
            statement_ids.len()
        );

        // Delete links first.
        sqlx::query("DELETE FROM statement_graph_link WHERE graph_id = ?1")
            .bind(graph_id.to_string())
            .execute(&mut *transaction)
            .await?;
        log::debug!("Deleted statement graph links for {graph_id}");

        Self::delete_unlinked_statements(&mut transaction, &statement_ids).await?;

        sqlx::query("DELETE FROM graphs WHERE graph_id = ?1")
            .bind(graph_id.to_string())
            .execute(&mut *transaction)
            .await?;
        log::debug!("Deleted graph {graph_id}");

        transaction.commit().await?;
        Ok(())
    }

    /// Deletes statements and credentials linked directly to a graph that no other graph
    /// links, while preserving the graph.
    pub async fn delete_graph_statements(&self, graph_id: &Uuid) -> Result<()> {
        log::debug!("Deleting statements for graph: {graph_id}");
        let mut transaction = self.begin_write().await?;
        let stmt_rows = sqlx::query(
            "SELECT DISTINCT statement_id FROM statement_graph_link WHERE graph_id = ?1",
        )
        .bind(graph_id.to_string())
        .fetch_all(&mut *transaction)
        .await?;

        let statement_ids: Vec<String> = stmt_rows
            .into_iter()
            .map(|row| row.get::<String, _>("statement_id"))
            .collect();

        sqlx::query("DELETE FROM statement_graph_link WHERE graph_id = ?1")
            .bind(graph_id.to_string())
            .execute(&mut *transaction)
            .await?;

        Self::delete_unlinked_statements(&mut transaction, &statement_ids).await?;
        transaction.commit().await?;
        Ok(())
    }

    /// Deletes those of the given statements and credentials that no graph links any more.
    ///
    /// Call this after removing graph links, in the same transaction. A credential is kept while its subject is still
    /// linked to a graph, and so is a credential about a DID or a DID statement, because
    /// exports find those by DID rather than through a graph.
    async fn delete_unlinked_statements(
        transaction: &mut Transaction<'_, SqliteDb>,
        statement_ids: &[String],
    ) -> Result<()> {
        if statement_ids.is_empty() {
            return Ok(());
        }

        let placeholders = vec!["?"; statement_ids.len()].join(", ");
        for table in GRAPH_OWNED_STATEMENT_TABLES {
            let q = format!(
                "DELETE FROM {table} WHERE id IN ({placeholders}) \
                 AND id NOT IN (SELECT statement_id FROM statement_graph_link)"
            );
            let mut sql = sqlx::query(&q);
            for statement_id in statement_ids {
                sql = sql.bind(statement_id);
            }
            sql.execute(&mut **transaction).await?;
            log::debug!("Deleted unlinked statements from {table}");
        }

        let q = format!(
            "DELETE FROM credential_statements WHERE id IN ({placeholders}) \
             AND id NOT IN (SELECT statement_id FROM statement_graph_link) \
             AND credential_subject NOT LIKE 'did:%' \
             AND credential_subject NOT IN (SELECT statement_id FROM statement_graph_link) \
             AND credential_subject NOT IN (SELECT id FROM did_statements)"
        );
        let mut sql = sqlx::query(&q);
        for statement_id in statement_ids {
            sql = sql.bind(statement_id);
        }
        sql.execute(&mut **transaction).await?;
        log::debug!("Deleted unlinked credentials");

        Ok(())
    }

    /// Returns a credential that `registered_by` already issued for `subject`.
    pub async fn find_credential(
        &self,
        subject: &str,
        registered_by: &str,
    ) -> Result<Option<Statement>> {
        Self::find_credential_with(&self.pool, subject, registered_by).await
    }

    async fn find_credential_with<'e>(
        executor: impl sqlx::SqliteExecutor<'e>,
        subject: &str,
        registered_by: &str,
    ) -> Result<Option<Statement>> {
        let row = sqlx::query(
            r#"
            SELECT statement FROM credential_statements
            WHERE credential_subject = ?1 AND registered_by = ?2
            ORDER BY id
            LIMIT 1
            "#,
        )
        .bind(subject)
        .bind(registered_by)
        .fetch_optional(executor)
        .await?;
        row.map(|row| {
            let statement: String = row.get("statement");
            serde_json::from_str(&statement).context("Failed to parse stored credential")
        })
        .transpose()
    }

    /// Registers `statement` and a credential for it in `graph_id`, in one transaction.
    ///
    /// If the credential's signer already has a credential for `statement`, that one is
    /// registered instead of `credential`. This covers another connection storing one after
    /// the caller checked with [`Self::find_credential`] and before this call. Returns the
    /// credential that was registered.
    pub(crate) async fn register_statement_with_credential(
        &self,
        statement: &Statement,
        credential: Statement,
        graph_id: &Uuid,
    ) -> Result<Statement> {
        let mut transaction = self.begin_write().await?;
        let credential = Self::find_credential_with(
            &mut *transaction,
            &statement.get_id(),
            credential.get_registered_by(),
        )
        .await?
        .unwrap_or(credential);
        for statement in [statement, &credential] {
            Self::register_statement_in_transaction(&mut transaction, statement, graph_id)
                .await
                .with_context(|| {
                    format!("Failed to register statement '{}'", statement.get_id())
                })?;
        }
        transaction.commit().await?;
        Ok(credential)
    }

    /// Registers a statement in the database, optionally associating it with a graph.
    pub async fn register_statement(&self, statement: &Statement, graph_id: &Uuid) -> Result<()> {
        log::trace!("Registering statement");
        let mut transaction = self.pool.begin().await?;
        Self::register_statement_in_transaction(&mut transaction, statement, graph_id).await?;
        transaction.commit().await?;
        Ok(())
    }

    /// Imports a set of statements atomically, avoiding one SQLite commit per statement.
    pub(crate) async fn register_statements_batch(
        &self,
        statements: impl IntoIterator<Item = Statement>,
        graph_id: &Uuid,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        for statement in statements {
            let statement_id = statement.get_id();
            Self::register_statement_in_transaction(&mut transaction, &statement, graph_id)
                .await
                .with_context(|| format!("Failed to register statement '{statement_id}'"))?;
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn register_statement_in_transaction(
        transaction: &mut Transaction<'_, SqliteDb>,
        statement: &Statement,
        graph_id: &Uuid,
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
                .execute(&mut **transaction)
                .await?;
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
                .execute(&mut **transaction)
                .await?;

                for data_item in s.data.to_vec_string() {
                    sqlx::query(
                        r#"
                      INSERT OR IGNORE INTO data_statement_subjects
                      (statement_id, subject) VALUES (?1, ?2)
                    "#,
                    )
                    .bind(&id)
                    .bind(data_item)
                    .execute(&mut **transaction)
                    .await?;
                }
            }
            Statement::MetadataRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
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
                .execute(&mut **transaction)
                .await?;
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
                .execute(&mut **transaction)
                .await?;
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
                .execute(&mut **transaction)
                .await?;

                for entity in s.entity.to_vec_string() {
                    sqlx::query(
                        r#"
                      INSERT OR IGNORE INTO entity_statement_subjects
                      (statement_id, entity) VALUES (?1, ?2)
                    "#,
                    )
                    .bind(&id)
                    .bind(entity)
                    .execute(&mut **transaction)
                    .await?;
                }
            }
            Statement::AssociationRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                let association_type = match s.r#type {
                    integrity::lineage::models::statements::AssociationType::Certifies => {
                        "certifies"
                    }
                    integrity::lineage::models::statements::AssociationType::Includes => "includes",
                    integrity::lineage::models::statements::AssociationType::IsInstanceOf => {
                        "isInstanceOf"
                    }
                };
                log::debug!("Registering association '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO association_statements
                    (id, statement, registered_by, subject, type) VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.subject)
                .bind(association_type)
                .execute(&mut **transaction)
                .await?;

                for item in &s.association {
                    sqlx::query(
                        r#"
                        INSERT OR IGNORE INTO association_statement_items
                        (statement_id, association_item) VALUES (?1, ?2)
                    "#,
                    )
                    .bind(&id)
                    .bind(item)
                    .execute(&mut **transaction)
                    .await?;
                }
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
                .execute(&mut **transaction)
                .await?;
            }
            Statement::CredentialSigstoreBundleRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering sigstore bundle '{id}'");
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO sigstore_statements
                    (id, statement, registered_by, subject) VALUES (?1, ?2, ?3, ?4)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&s.subject)
                .execute(&mut **transaction)
                .await?;
            }
            Statement::CredentialRegistration(s) => {
                let statement = serde_json::to_value(statement)?;
                let id = s.get_id();
                let statement_data = serde_json::to_string(&statement)?;
                log::debug!("Registering credential '{id}'");
                let subject = s.credential.credential_subjects.first().ok_or_else(|| {
                    anyhow::anyhow!("Credential registration has no credential subjects")
                })?;
                let subject_json = serde_json::to_value(subject)
                    .context("Failed to serialize credential subject")?;
                let subject_id = extract_credential_subject_id(&subject_json).context(format!(
                    "Failed to extract subject ID from credential registration '{id}'"
                ))?;

                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO credential_statements
                    (id, statement, registered_by, credential_subject) VALUES (?1, ?2, ?3, ?4)
                "#,
                )
                .bind(&id)
                .bind(&statement_data)
                .bind(&s.registered_by)
                .bind(&subject_id)
                .execute(&mut **transaction)
                .await?;
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
                .execute(&mut **transaction)
                .await?;
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
                .execute(&mut **transaction)
                .await?;
            }
        }
        Self::associate_statement_to_graph_in_transaction(
            transaction,
            &statement.get_id(),
            graph_id,
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn associate_statement_to_graph_in_transaction(
        transaction: &mut Transaction<'_, SqliteDb>,
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
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    /// Retrieves statements explicitly linked to this graph via `statement_graph_link`,
    /// regardless of whether anything else in the graph references them.
    async fn retrieve_directly_linked_statements(
        &self,
        graph_id: &Uuid,
    ) -> Result<Vec<sqlx::sqlite::SqliteRow>> {
        let query = r#"
            SELECT statement, NULL as metadata, NULL as vc, NULL as did
            FROM data_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM metadata_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM storage_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM entity_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM association_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM governance_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM credential_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM dsse_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM sigstore_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
            UNION ALL
            SELECT statement, NULL, NULL, NULL
            FROM did_statements s JOIN statement_graph_link l ON s.id = l.statement_id
            WHERE l.graph_id = ?1
        "#;

        sqlx::query(query)
            .bind(graph_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Retrieves the statements associated to the graph ID.
    ///
    /// Returns the graph with its statements populated, including statements
    /// from parent graphs in the hierarchy.
    pub async fn retrieve_statements(&self, graph_id: &Uuid) -> Result<Vec<Statement>> {
        log::info!("Retrieving statements for graph {graph_id:?}");

        let direct_rows = self.retrieve_directly_linked_statements(graph_id).await?;
        log::debug!("Found '{}' directly linked statements", direct_rows.len());
        let mut statements = rows_to_statements(direct_rows)?;

        let compute_query_str = format!(
            r#"
            WITH RECURSIVE {GRAPH_HIERARCHY_CTE}
            SELECT
              s.statement
              , vc.statement as vc
              , metadata.statement as metadata
              , did.statement as did
            FROM computation_statements s
            LEFT JOIN statement_graph_link l ON s.id = l.statement_id
            LEFT JOIN graphs g ON l.graph_id = g.graph_id
            LEFT JOIN credential_statements vc
              ON vc.credential_subject = s.id AND {CREDENTIAL_IN_HIERARCHY}
            LEFT JOIN metadata_statements metadata on metadata.subject = s.id
            LEFT JOIN did_statements did ON s.registered_by = did.did
            WHERE g.graph_id = ?1
        "#
        );

        let compute_rows = sqlx::query(&compute_query_str)
            .bind(graph_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        if compute_rows.is_empty() {
            log::info!("No computation statements found for graph(s) {graph_id:?}");
            self.get_global_statements(graph_id, &mut statements)
                .await?;
            return Ok(statements.into_values().collect());
        }

        let mut subjects: Vec<String> = Vec::new();

        log::debug!("Found '{}' compute statements", compute_rows.len());
        statements.extend(rows_to_statements(compute_rows)?);
        for statement in statements.values() {
            if let Statement::ComputationRegistration(s) = statement {
                subjects.extend(s.input.to_vec_string());
                subjects.extend(s.output.to_vec_string());
            }
        }

        log::debug!("Getting statements for subjects: {subjects:?}");
        let rows = self
            .retrieve_related_statements(graph_id, &subjects)
            .await?;
        log::debug!("Found '{}' related statements", rows.len());
        statements.extend(rows_to_statements(rows)?);

        let association_targets: Vec<String> = statements
            .values()
            .filter_map(|statement| match statement {
                Statement::AssociationRegistration(s) => Some(s.association.clone()),
                _ => None,
            })
            .flatten()
            .collect();

        if !association_targets.is_empty() {
            log::debug!("Getting statements for association targets: {association_targets:?}");
            let association_rows = self
                .retrieve_related_statements(graph_id, &association_targets)
                .await?;
            log::debug!(
                "Found '{}' related statements for association targets",
                association_rows.len()
            );
            statements.extend(rows_to_statements(association_rows)?);
        }

        self.get_global_statements(graph_id, &mut statements)
            .await?;

        Ok(statements.into_values().collect())
    }

    /// Returns the graph ancestry ordered from root to the provided graph.
    pub async fn get_graph_ancestors(&self, graph_id: &Uuid) -> Result<Vec<Context>> {
        log::info!("Getting ancestors of graph {graph_id}");

        let rows: Vec<Context> = sqlx::query_as(
            r#"
            WITH RECURSIVE ancestors AS (
                SELECT graph_id, name, parent_id, 0 as depth
                FROM graphs
                WHERE graph_id = ?1

                UNION ALL

                SELECT g.graph_id, g.name, g.parent_id, a.depth + 1
                FROM graphs g
                INNER JOIN ancestors a ON g.graph_id = a.parent_id
            )
            SELECT graph_id, name, parent_id
            FROM ancestors
            ORDER BY depth DESC
            "#,
        )
        .bind(graph_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        log::info!("{} generations in {graph_id}", rows.len());

        Ok(rows)
    }

    /// Deletes all data from the database
    pub async fn purge(&self) -> Result<()> {
        // Drop all tables dynamically to avoid falling out of sync with schema changes.
        sqlx::query("PRAGMA foreign_keys = OFF;")
            .execute(&self.pool)
            .await?;

        let table_rows = sqlx::query(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in table_rows {
            let name: String = row.try_get("name")?;
            let drop_sql = format!("DROP TABLE IF EXISTS \"{}\";", name.replace('\"', "\"\""));
            sqlx::query(&drop_sql).execute(&self.pool).await?;
        }

        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&self.pool)
            .await?;

        self.init().await?;
        Ok(())
    }

    async fn get_global_statements(
        &self,
        graph_id: &Uuid,
        statements: &mut HashMap<String, Statement>,
    ) -> Result<()> {
        // Get the Credential, CredSigStore and DID statements for all the previously fetched
        // statements. Credentials come from this graph and its ancestors (see
        // CREDENTIAL_IN_HIERARCHY); the rest come from any graph.
        let mut dids = HashSet::new();
        let mut credential_subjects = HashSet::new();
        let mut referenced_cids = HashSet::new();

        for stmt in statements.values() {
            dids.insert(stmt.get_registered_by().to_owned());
            credential_subjects.insert(stmt.get_id().to_owned());
            referenced_cids.extend(stmt.referenced_cids());
        }

        let mut credential_lookup_subjects = credential_subjects.clone();
        credential_lookup_subjects.extend(dids.iter().cloned());

        log::debug!("Getting credential statements for subjects: {credential_lookup_subjects:?}");
        // ?1 is the graph ID, so the subjects start at ?2.
        let placeholders = (2..credential_lookup_subjects.len() + 2)
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let global_query = format!(
            r#"
            WITH RECURSIVE {GRAPH_HIERARCHY_CTE}
            SELECT statement, NULL as metadata, NULL as vc, NULL as did
            FROM credential_statements vc
            WHERE credential_subject IN ({placeholders}) AND {CREDENTIAL_IN_HIERARCHY}
        "#
        );

        let mut sql_query = sqlx::query(&global_query).bind(graph_id.to_string());
        for credential_subject in &credential_lookup_subjects {
            sql_query = sql_query.bind(credential_subject);
        }

        let vc_rows = sql_query.fetch_all(&self.pool).await?;
        log::debug!("Found '{}' credential statements", vc_rows.len());

        let vc_statements = rows_to_statements(vc_rows)?;
        statements.extend(vc_statements);

        if !referenced_cids.is_empty() {
            log::debug!("Getting sigstore statements for subjects: {referenced_cids:?}");
            let placeholders = vec!["?"; referenced_cids.len()].join(", ");
            let sigstore_query = format!(
                r#"
                SELECT statement, NULL as metadata, NULL as vc, NULL as did
                FROM sigstore_statements
                WHERE subject IN ({})
            "#,
                placeholders
            );

            let mut sql_query = sqlx::query(&sigstore_query);
            for referenced_cid in &referenced_cids {
                sql_query = sql_query.bind(referenced_cid);
            }

            let sigstore_rows = sql_query.fetch_all(&self.pool).await?;
            log::debug!("Found '{}' sigstore statements", sigstore_rows.len());

            let sigstore_statements = rows_to_statements(sigstore_rows)?;
            statements.extend(sigstore_statements);
        }

        if !dids.is_empty() {
            log::debug!("Getting DID statements for subjects: {dids:?}");
            let placeholders = vec!["?"; dids.len()].join(", ");
            let global_query = format!(
                r#"
                SELECT
                  did.statement as statement
                  ,meta.statement as metadata
                  ,vc.statement as vc
                  ,NULL as did
                FROM did_statements did
                LEFT JOIN metadata_statements meta
                  ON did.did = meta.subject OR did.id = meta.subject
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

            let did_statements = rows_to_statements(did_rows)?;
            statements.extend(did_statements);
        }

        Ok(())
    }
}
