use std::collections::{HashMap, HashSet};

use anyhow::Result;
use integrity::lineage::models::statements::{Statement, StatementTrait};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{rows_to_statements, Context};

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
            LEFT JOIN association_statement_items asi ON association.id = asi.statement_id
            LEFT JOIN entity_statements entity ON sgl.statement_id = entity.id
            LEFT JOIN entity_statement_subjects ess ON entity.id = ess.statement_id
            WHERE dss.subject IN {}
               OR metadata.subject IN {}
               OR storage.data IN {}
               OR asi.association_item IN {}
               OR association.subject IN {}
               OR ess.entity IN {}
            ORDER BY gh.level;
            "#,
            in_clause, in_clause, in_clause, in_clause, in_clause, in_clause
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

    /// Deletes a graph and all descendant graphs, along with statements linked to those graphs.
    pub async fn delete_graph_tree(&self, graph_id: &Uuid) -> Result<()> {
        // Get all graph ids in the subtree (including the root).
        log::debug!("Deleting graph tree for {graph_id}");
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
        .fetch_all(&self.pool)
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
        let stmt_rows = stmt_sql.fetch_all(&self.pool).await?;
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
        link_sql.execute(&self.pool).await?;
        log::debug!("Deleted statement graph links");

        if !statement_ids.is_empty() {
            let stmt_placeholders = vec!["?"; statement_ids.len()].join(", ");
            let tables = [
                "computation_statements",
                "data_statements",
                "metadata_statements",
                "storage_statements",
                "entity_statements",
                "association_statements",
            ];

            for table in tables {
                let q = format!("DELETE FROM {} WHERE id IN ({})", table, stmt_placeholders);
                let mut sql = sqlx::query(&q);
                for sid in &statement_ids {
                    sql = sql.bind(sid);
                }
                sql.execute(&self.pool).await?;
                log::debug!("Deleted statements from {table}");
            }
        }

        // Delete graphs (disable FK checks to avoid parent/child ordering issues).
        sqlx::query("PRAGMA foreign_keys = OFF;")
            .execute(&self.pool)
            .await?;
        let graph_del_query = format!("DELETE FROM graphs WHERE graph_id IN ({})", placeholders);
        let mut graph_del_sql = sqlx::query(&graph_del_query);
        for gid in &graph_ids {
            graph_del_sql = graph_del_sql.bind(gid);
        }
        graph_del_sql.execute(&self.pool).await?;
        log::debug!("Deleted graphs");
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Deletes a graph and its statements only if it has no child graphs.
    pub async fn delete_graph_no_children(&self, graph_id: &Uuid) -> Result<()> {
        log::debug!("Deleting graph without children: {graph_id}");
        let child_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM graphs
            WHERE parent_id = ?1
            "#,
        )
        .bind(graph_id.to_string())
        .fetch_one(&self.pool)
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
        .fetch_all(&self.pool)
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
            .execute(&self.pool)
            .await?;
        log::debug!("Deleted statement graph links for {graph_id}");

        if !statement_ids.is_empty() {
            let stmt_placeholders = vec!["?"; statement_ids.len()].join(", ");
            let tables = [
                "computation_statements",
                "data_statements",
                "metadata_statements",
                "storage_statements",
                "entity_statements",
                "association_statements",
            ];

            for table in tables {
                let q = format!("DELETE FROM {} WHERE id IN ({})", table, stmt_placeholders);
                let mut sql = sqlx::query(&q);
                for sid in &statement_ids {
                    sql = sql.bind(sid);
                }
                sql.execute(&self.pool).await?;
                log::debug!("Deleted statements from {table}");
            }
        }

        sqlx::query("DELETE FROM graphs WHERE graph_id = ?1")
            .bind(graph_id.to_string())
            .execute(&self.pool)
            .await?;
        log::debug!("Deleted graph {graph_id}");

        Ok(())
    }

    /// Registers a statement in the database, optionally associating it with a graph.
    ///
    /// Graph-specific statements (computation, data, metadata, etc.) are linked to the
    /// provided graph_id. Global statements (credentials, DIDs) are stored without graph association.
    pub async fn register_statement(&self, statement: &Statement, graph_id: &Uuid) -> Result<()> {
        log::trace!("Registering statement");
        match statement {
            Statement::AssociationRegistration(_)
            | Statement::ComputationRegistration(_)
            | Statement::DataRegistration(_)
            | Statement::EntityRegistration(_)
            | Statement::GovernanceRegistration(_)
            | Statement::MetadataRegistration(_)
            | Statement::StorageRegistration(_) => {
                self.register_graph_statement(statement, graph_id).await
            }
            Statement::CredentialDsseRegistration(_)
            | Statement::CredentialRegistration(_)
            | Statement::CredentialSigstoreBundleRegistration(_)
            | Statement::DidRegistration(_) => self.register_global_statement(statement).await,
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

    /// Retrieves the statements associated to the graph ID.
    ///
    /// Returns the graph with its statements populated, including statements
    /// from parent graphs in the hierarchy.
    pub async fn retrieve_statements(&self, graph_id: &Uuid) -> Result<Vec<Statement>> {
        log::info!("Retrieving statements for graph {graph_id:?}");

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
            return Ok(vec![]);
        }

        let mut subjects: Vec<String> = Vec::new();

        log::debug!("Found '{}' compute statements", compute_rows.len());
        let mut statements = rows_to_statements(compute_rows)?;
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

        let association_subjects: Vec<String> = statements
            .values()
            .filter_map(|statement| match statement {
                Statement::AssociationRegistration(s) => Some(s.association.clone()),
                _ => None,
            })
            .flatten()
            .collect();

        if !association_subjects.is_empty() {
            log::debug!("Getting statements for association targets: {association_subjects:?}");
            let association_rows = self
                .retrieve_related_statements(graph_id, &association_subjects)
                .await?;
            log::debug!(
                "Found '{}' related statements for association targets",
                association_rows.len()
            );
            statements.extend(rows_to_statements(association_rows)?);
        }

        self.get_global_statements(&mut statements).await?;

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

    /// Used to register statements associtated with a specific graph_id (aka NON-Global)
    async fn register_graph_statement(&self, statement: &Statement, graph_id: &Uuid) -> Result<()> {
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

                self.associate_statement_to_graph(&id, graph_id).await
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

                self.associate_statement_to_graph(&id, graph_id).await?;

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

                self.associate_statement_to_graph(&id, graph_id).await
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

                self.associate_statement_to_graph(&id, graph_id).await
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

                self.associate_statement_to_graph(&id, graph_id).await?;

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
                .execute(&self.pool)
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
                    .execute(&self.pool)
                    .await?;
                }

                self.associate_statement_to_graph(&id, graph_id).await
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
            Statement::CredentialSigstoreBundleRegistration(_)
            | Statement::DidRegistration(_)
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
            Statement::ComputationRegistration(_)
            | Statement::AssociationRegistration(_)
            | Statement::DataRegistration(_)
            | Statement::MetadataRegistration(_)
            | Statement::StorageRegistration(_)
            | Statement::GovernanceRegistration(_)
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
        // Get the Credential, CredDsse, CredSigStore, DID Statements
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
            SELECT statement, NULL as metadata, NULL as vc, NULL as did
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

        let vc_statements = rows_to_statements(vc_rows)?;
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
                  ,NULL as did
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

            let did_statements = rows_to_statements(did_rows)?;
            statements.extend(did_statements);
        }

        Ok(())
    }
}
