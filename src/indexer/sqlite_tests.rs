#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use anyhow::Result;
    use integrity::lineage::models::{
        manifest::Manifest,
        statements::{
            AssociationStatement, AssociationType, DataStatement, Statement, StatementTrait,
        },
    };
    use pyo3::{PyErr, Python};
    use pyo3_async_runtimes::tokio::get_runtime;
    use sqlx::Row;
    use tempfile::tempdir;
    use uuid::Uuid;

    use crate::{
        config::Config,
        indexer::{Context, Sqlite},
    };

    async fn setup_db() -> Result<Sqlite> {
        let db = Sqlite::new("sqlite::memory:").await?;
        db.init().await?;
        Ok(db)
    }

    async fn graph_count(db: &Sqlite) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM graphs")
            .fetch_one(db.pool())
            .await?;
        Ok(row.get::<i64, _>("count"))
    }

    async fn data_statement_count(db: &Sqlite) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM data_statements")
            .fetch_one(db.pool())
            .await?;
        Ok(row.get::<i64, _>("count"))
    }

    fn test_manifest_path() -> &'static Path {
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/indexer/testdata/simple.json"
        ))
    }

    fn with_test_config<T>(
        app_dir: &Path,
        f: impl FnOnce(Python<'_>, Config) -> Result<T>,
    ) -> Result<T> {
        Python::initialize();
        Python::attach(|py| {
            py.detach(|| get_runtime().block_on(Config::reset_internal()))?;
            let cfg =
                py.detach(|| get_runtime().block_on(Config::init(app_dir.to_path_buf(), None)))?;

            let result = f(py, cfg);

            py.detach(|| get_runtime().block_on(Config::reset_internal()))?;
            result
        })
    }

    #[tokio::test]
    async fn test_create_graph_inserts_parent() -> Result<()> {
        let db = setup_db().await?;
        let parent_id = Uuid::new_v4();
        let graph = Context {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(parent_id),
        };

        db.create_graph(&graph).await?;

        let row = sqlx::query("SELECT COUNT(*) as count FROM graphs")
            .fetch_one(db.pool())
            .await?;
        let count = row.get::<i64, _>("count");
        // child + parent
        assert_eq!(count, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_graph_ancestors_order() -> Result<()> {
        let db = setup_db().await?;
        let root = Context {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Context {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(root.id),
        };
        db.create_graph(&child).await?;

        let grandchild = Context {
            id: Uuid::new_v4(),
            name: "grandchild".to_string(),
            parent: Some(child.id),
        };
        db.create_graph(&grandchild).await?;

        let ancestors = db.get_graph_ancestors(&grandchild.id).await?;
        assert_eq!(ancestors.len(), 3);
        for window in ancestors.windows(2) {
            let current = &window[0];
            let next = &window[1];
            assert_eq!(next.parent, Some(current.id));
        }
        assert_eq!(ancestors.first().unwrap().parent, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_graph_no_children() -> Result<()> {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "solo".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let data = DataStatement::create(
            vec!["urn:cid:input1".to_string()],
            "did:key:tester".to_string(),
            None,
        )
        .await?;
        let statement = Statement::DataRegistration(data);
        db.register_statement(&statement, &graph.id).await?;

        assert_eq!(data_statement_count(&db).await?, 1);
        db.delete_graph_no_children(&graph.id).await?;
        assert_eq!(graph_count(&db).await?, 0);
        assert_eq!(data_statement_count(&db).await?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_graph_no_children_errors_on_child() -> Result<()> {
        let db = setup_db().await?;
        let root = Context {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Context {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(root.id),
        };
        db.create_graph(&child).await?;

        let err = db.delete_graph_no_children(&root.id).await;
        assert!(err.is_err());
        assert_eq!(graph_count(&db).await?, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_graph_tree() -> Result<()> {
        let db = setup_db().await?;
        let root = Context {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Context {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(root.id),
        };
        db.create_graph(&child).await?;

        let grandchild = Context {
            id: Uuid::new_v4(),
            name: "grandchild".to_string(),
            parent: Some(child.id),
        };
        db.create_graph(&grandchild).await?;

        db.delete_graph_tree(&root.id).await?;
        assert_eq!(graph_count(&db).await?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_purge_reinitializes_schema() -> Result<()> {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;
        assert_eq!(graph_count(&db).await?, 1);

        db.purge().await?;
        assert_eq!(graph_count(&db).await?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_association_statement_links_items() -> Result<()> {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "assoc-graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let association = vec![
            "urn:cid:item1".to_string(),
            "urn:cid:item2".to_string(),
            "urn:cid:item3".to_string(),
        ];
        let statement = AssociationStatement::create(
            "urn:cid:subject".to_string(),
            association.clone(),
            AssociationType::Includes,
            "did:key:tester".to_string(),
            None,
        )
        .await?;
        let statement = Statement::AssociationRegistration(statement);
        db.register_statement(&statement, &graph.id).await?;

        let row = sqlx::query("SELECT COUNT(*) as count FROM association_statements")
            .fetch_one(db.pool())
            .await?;
        let count = row.get::<i64, _>("count");
        assert_eq!(count, 1);

        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM association_statement_items WHERE statement_id = ?1",
        )
        .bind(statement.get_id())
        .fetch_one(db.pool())
        .await?;
        let item_count = row.get::<i64, _>("count");
        assert_eq!(item_count, association.len() as i64);

        let mut rows = sqlx::query(
            r#"
            SELECT association_item
            FROM association_statement_items
            WHERE statement_id = ?1
            ORDER BY association_item
            "#,
        )
        .bind(statement.get_id())
        .fetch_all(db.pool())
        .await?;

        let mut items: Vec<String> = rows
            .drain(..)
            .map(|row| row.get::<String, _>("association_item"))
            .collect();
        items.sort();
        let mut expected = association;
        expected.sort();
        assert_eq!(items, expected);

        Ok(())
    }

    #[test]
    fn test_context_import_manifest_imports_statements_and_blobs() -> Result<()> {
        let temp_dir = tempdir()?;
        let manifest_path = test_manifest_path().to_path_buf();
        let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;

        with_test_config(temp_dir.path(), |py, cfg| {
            let context = Context {
                id: Uuid::new_v4(),
                name: "import-target".to_string(),
                parent: None,
            };

            context.import_manifest(py, manifest_path.clone())?;

            let statements = py
                .detach(|| get_runtime().block_on(cfg.sql_lite.retrieve_statements(&context.id)))?;

            assert_eq!(statements.len(), manifest.statements.len());
            for statement_id in manifest.statements.keys() {
                assert!(statements
                    .iter()
                    .any(|statement| statement.get_id() == *statement_id));
            }

            for blob_cid in manifest.blobs.keys() {
                assert!(cfg.app_dir.join("blobs").join(blob_cid).exists());
            }

            Ok(())
        })
    }

    #[test]
    fn test_context_import_manifest_errors_for_missing_path() -> Result<()> {
        let temp_dir = tempdir()?;
        let missing_path = temp_dir.path().join("missing-manifest.json");

        with_test_config(temp_dir.path(), |py, _cfg| {
            let context = Context {
                id: Uuid::new_v4(),
                name: "import-target".to_string(),
                parent: None,
            };

            let err: PyErr = context
                .import_manifest(py, missing_path.clone())
                .unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("Failed to open manifest file"));
            assert!(msg.contains("No such file") || msg.contains("os error 2"));

            Ok(())
        })
    }

    #[test]
    fn test_context_import_manifest_errors_for_bad_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let bad_manifest_path = temp_dir.path().join("bad-manifest.json");
        fs::write(&bad_manifest_path, "{ definitely not valid json")?;

        with_test_config(temp_dir.path(), |py, _cfg| {
            let context = Context {
                id: Uuid::new_v4(),
                name: "import-target".to_string(),
                parent: None,
            };

            let err: PyErr = context
                .import_manifest(py, bad_manifest_path.clone())
                .unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("Failed to deserialize manifest from file"));
            assert!(msg.contains("bad-manifest.json"));

            Ok(())
        })
    }
}
