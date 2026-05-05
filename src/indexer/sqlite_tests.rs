#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use anyhow::Result;
    use integrity::lineage::models::{
        manifest::Manifest,
        statements::{
            AssociationStatement, AssociationType, ComputationStatement, DataStatement, Statement,
            StatementTrait,
        },
    };
    use pyo3::{PyErr, Python};
    use pyo3_async_runtimes::tokio::get_runtime;
    use serde_json::json;
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
    async fn test_sigstore_table_has_subject_column() -> Result<()> {
        let db = setup_db().await?;

        let rows = sqlx::query("PRAGMA table_info(sigstore_statements)")
            .fetch_all(db.pool())
            .await?;
        let columns: Vec<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        assert!(columns.contains(&"subject".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_init_migrates_legacy_sigstore_table() -> Result<()> {
        let db = Sqlite::new("sqlite::memory:").await?;
        sqlx::query(
            r#"
            CREATE TABLE sigstore_statements (
                id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                registered_by TEXT NOT NULL
            )
            "#,
        )
        .execute(db.pool())
        .await?;

        db.init().await?;

        let rows = sqlx::query("PRAGMA table_info(sigstore_statements)")
            .fetch_all(db.pool())
            .await?;
        let columns: Vec<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();
        assert!(columns.contains(&"subject".to_string()));

        let indexes = sqlx::query("PRAGMA index_list(sigstore_statements)")
            .fetch_all(db.pool())
            .await?;
        let index_names: Vec<String> = indexes
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();
        assert!(index_names.contains(&"idx_sigstore_statements_subject".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_statements_includes_sigstore_bundle_by_subject_reference() -> Result<()>
    {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "sigstore-graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let subject = "urn:cid:bagaachra62qpuplhnpw24ff33lsehggwjmh5am3eibgovtrkrwhn6nmnjkeq";
        let sigstore_id = "urn:cid:bagb6qaq6edu6bmyvf5usnk7s3wrscjzvqjio5m2aygz3gwdqo2bxshfjbzs5s";
        let sigstore_statement = json!({
            "@context": "urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m",
            "@id": sigstore_id,
            "@type": "CredentialRegistration",
            "registeredBy": "did:key:tester",
            "sigstoreBundle": "e30=",
            "subject": subject,
            "timestamp": "2026-03-24T19:48:02Z"
        });

        sqlx::query(
            r#"
            INSERT INTO sigstore_statements (id, statement, registered_by, subject)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(sigstore_id)
        .bind(sigstore_statement.to_string())
        .bind("did:key:tester")
        .bind(subject)
        .execute(db.pool())
        .await?;

        let computation_statement = Statement::ComputationRegistration(
            ComputationStatement::create(
                None,
                vec![subject.to_string()],
                vec![
                    "urn:cid:bafkr4ien5sae7ddq6tys5ybcm2g7lhzj6ciyq6f4z6nsd7iqbmdy6s7h3u"
                        .to_string(),
                ],
                "did:key:tester".to_string(),
                None,
                "did:key:tester".to_string(),
                None,
            )
            .await?,
        );
        db.register_statement(&computation_statement, &graph.id)
            .await?;

        let statements = db.retrieve_statements(&graph.id).await?;
        let statement_ids: Vec<String> = statements.into_iter().map(|s| s.get_id()).collect();

        assert!(statement_ids.contains(&sigstore_id.to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_statements_includes_did_metadata_by_statement_id() -> Result<()> {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "did-metadata-graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let computation_id = "urn:cid:bagb6qaq6edidmeta0000000000000000000000000000000000000000001";
        let did_statement_id =
            "urn:cid:bagb6qaq6edidmeta0000000000000000000000000000000000000000002";
        let metadata_id = "urn:cid:bagb6qaq6edidmeta0000000000000000000000000000000000000000003";
        let did = "did:key:tester";

        let computation_statement = format!(
            r#"{{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"{computation_id}","@type":"ComputationRegistration","input":"urn:cid:bafkr4ididmetainput00000000000000000000000000000000000000000","operatedBy":"{did}","output":"urn:cid:bafkr4ididmetaoutput000000000000000000000000000000000000000","registeredBy":"{did}","timestamp":"2026-03-18T15:35:04Z"}}"#
        );
        let did_statement = format!(
            r#"{{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"{did_statement_id}","@type":"DidRegistration","did":"{did}","registeredBy":"{did}","timestamp":"2026-03-18T15:35:04Z"}}"#
        );
        let metadata_statement = format!(
            r#"{{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"{metadata_id}","@type":"MetadataRegistration","metadata":"urn:cid:baga6yaq6edidmetadata000000000000000000000000000000000000000","registeredBy":"{did}","subject":"{did_statement_id}","timestamp":"2026-03-18T15:35:04Z"}}"#
        );

        sqlx::query(
            r#"
            INSERT INTO computation_statements (id, statement, registered_by)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(computation_id)
        .bind(&computation_statement)
        .bind(did)
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO did_statements (id, statement, registered_by, type, did)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(did_statement_id)
        .bind(&did_statement)
        .bind(did)
        .bind("regular")
        .bind(did)
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO metadata_statements (id, statement, registered_by, subject)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(metadata_id)
        .bind(&metadata_statement)
        .bind(did)
        .bind(did_statement_id)
        .execute(db.pool())
        .await?;

        db.associate_statement_to_graph(computation_id, &graph.id)
            .await?;

        let statements = db.retrieve_statements(&graph.id).await?;

        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == did_statement_id));
        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == metadata_id));

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

    #[tokio::test]
    async fn test_retrieve_statements_includes_association_matching_subject() -> Result<()> {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "assoc-subject-graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let computation_id =
            "urn:cid:bagb6qaq6eazzw7iq6lvukorvca2eg6cmo37rsxazphv57stzfawbkjmsojkha";
        let association_id =
            "urn:cid:bagb6qaq6edt4b3nvnds2zzqmig7qi3lksztebn5nexm34bp3zil4xu7pnc7wg";
        let matching_subject =
            "urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i";
        let non_matching_item =
            "urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue";

        let computation_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6eazzw7iq6lvukorvca2eg6cmo37rsxazphv57stzfawbkjmsojkha","@type":"ComputationRegistration","input":"urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i","operatedBy":"did:key:tester","output":"urn:cid:bafkr000000000000000000000000000000000000000000000000000000","registeredBy":"did:key:tester","timestamp":"2026-03-18T15:35:04Z"}"#;
        let association_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6edt4b3nvnds2zzqmig7qi3lksztebn5nexm34bp3zil4xu7pnc7wg","@type":"AssociationRegistration","association":["urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue"],"registeredBy":"did:key:tester","subject":"urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i","timestamp":"2026-03-18T15:35:04Z","type":"includes"}"#;

        sqlx::query(
            r#"
            INSERT INTO computation_statements (id, statement, registered_by)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(computation_id)
        .bind(computation_statement)
        .bind("did:key:tester")
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO association_statements (id, statement, registered_by, subject, type)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(association_id)
        .bind(association_statement)
        .bind("did:key:tester")
        .bind(matching_subject)
        .bind("includes")
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO association_statement_items (statement_id, association_item)
            VALUES (?1, ?2)
            "#,
        )
        .bind(association_id)
        .bind(non_matching_item)
        .execute(db.pool())
        .await?;

        db.associate_statement_to_graph(computation_id, &graph.id)
            .await?;
        db.associate_statement_to_graph(association_id, &graph.id)
            .await?;

        let statements = db.retrieve_statements(&graph.id).await?;

        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == computation_id));
        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == association_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_statements_includes_association_target_data_and_metadata() -> Result<()>
    {
        let db = setup_db().await?;
        let graph = Context {
            id: Uuid::new_v4(),
            name: "assoc-target-graph".to_string(),
            parent: None,
        };
        db.create_graph(&graph).await?;

        let computation_id =
            "urn:cid:bagb6qaq6eazzw7iq6lvukorvca2eg6cmo37rsxazphv57stzfawbkjmsojkha";
        let association_id =
            "urn:cid:bagb6qaq6edt4b3nvnds2zzqmig7qi3lksztebn5nexm34bp3zil4xu7pnc7wg";
        let associated_data_id =
            "urn:cid:bagb6qaq6ebtq6zlas2qntunzfsnrmjgtxepege3je3ndc6wgvwicuhhvtc3uy";
        let associated_metadata_id =
            "urn:cid:bagb6qaq6ebm22azch76xgdokrebgfvliv4rorlmy2dvzsxh6dhghzgcg2p6as";
        let association_subject =
            "urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i";
        let associated_item = "urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue";

        let computation_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6eazzw7iq6lvukorvca2eg6cmo37rsxazphv57stzfawbkjmsojkha","@type":"ComputationRegistration","input":"urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i","operatedBy":"did:key:tester","output":"urn:cid:bafkr000000000000000000000000000000000000000000000000000000","registeredBy":"did:key:tester","timestamp":"2026-03-18T15:35:04Z"}"#;
        let association_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6edt4b3nvnds2zzqmig7qi3lksztebn5nexm34bp3zil4xu7pnc7wg","@type":"AssociationRegistration","association":["urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue"],"registeredBy":"did:key:tester","subject":"urn:cid:bafkr4ieb6ekctdq2jyexznx7hqfaiobouebz6b3ngrekb4sskyexs65z2i","timestamp":"2026-03-18T15:35:04Z","type":"includes"}"#;
        let associated_data_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6ebtq6zlas2qntunzfsnrmjgtxepege3je3ndc6wgvwicuhhvtc3uy","@type":"DataRegistration","data":"urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue","registeredBy":"did:key:tester","timestamp":"2026-03-18T15:35:04Z"}"#;
        let associated_metadata_statement = r#"{"@context":"urn:cid:bafkr4ic7ydwk3rtoltyzx4zn3vvu3r7hpzxtmbzmnksotx7k5nbnwclf6m","@id":"urn:cid:bagb6qaq6ebm22azch76xgdokrebgfvliv4rorlmy2dvzsxh6dhghzgcg2p6as","@type":"MetadataRegistration","metadata":"urn:cid:baga6yaq6ebxlzahvg745rbh35ugkkwm3xd2htul5axb6pbhqk4zqt2fqlhpza","registeredBy":"did:key:tester","subject":"urn:cid:bafkr4ibguhm4o5xd633dvhcbykybwc2iekwlvfz76s5hcqmdfifnpxkdue","timestamp":"2026-03-18T15:35:04Z"}"#;

        sqlx::query(
            r#"
            INSERT INTO computation_statements (id, statement, registered_by)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(computation_id)
        .bind(computation_statement)
        .bind("did:key:tester")
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO association_statements (id, statement, registered_by, subject, type)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(association_id)
        .bind(association_statement)
        .bind("did:key:tester")
        .bind(association_subject)
        .bind("includes")
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO association_statement_items (statement_id, association_item)
            VALUES (?1, ?2)
            "#,
        )
        .bind(association_id)
        .bind(associated_item)
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO data_statements (id, statement, registered_by)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(associated_data_id)
        .bind(associated_data_statement)
        .bind("did:key:tester")
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO data_statement_subjects (statement_id, subject)
            VALUES (?1, ?2)
            "#,
        )
        .bind(associated_data_id)
        .bind(associated_item)
        .execute(db.pool())
        .await?;

        sqlx::query(
            r#"
            INSERT INTO metadata_statements (id, statement, registered_by, subject)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(associated_metadata_id)
        .bind(associated_metadata_statement)
        .bind("did:key:tester")
        .bind(associated_item)
        .execute(db.pool())
        .await?;

        db.associate_statement_to_graph(computation_id, &graph.id)
            .await?;
        db.associate_statement_to_graph(association_id, &graph.id)
            .await?;
        db.associate_statement_to_graph(associated_data_id, &graph.id)
            .await?;
        db.associate_statement_to_graph(associated_metadata_id, &graph.id)
            .await?;

        let statements = db.retrieve_statements(&graph.id).await?;

        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == association_id));
        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == associated_data_id));
        assert!(statements
            .iter()
            .any(|statement| statement.get_id() == associated_metadata_id));

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
