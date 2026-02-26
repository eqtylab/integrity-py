#[cfg(test)]
mod tests {
    use anyhow::Result;
    use integrity::lineage::models::statements::{DataStatement, Statement};
    use sqlx::Row;
    use uuid::Uuid;

    use crate::indexer::{Graph, Sqlite};

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

    #[tokio::test]
    async fn test_create_graph_inserts_parent() -> Result<()> {
        let db = setup_db().await?;
        let parent_id = Uuid::new_v4();
        let graph = Graph {
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
        let root = Graph {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Graph {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(root.id),
        };
        db.create_graph(&child).await?;

        let grandchild = Graph {
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
        let graph = Graph {
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
        let root = Graph {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Graph {
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
        let root = Graph {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            parent: None,
        };
        db.create_graph(&root).await?;

        let child = Graph {
            id: Uuid::new_v4(),
            name: "child".to_string(),
            parent: Some(root.id),
        };
        db.create_graph(&child).await?;

        let grandchild = Graph {
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
        let graph = Graph {
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
}
