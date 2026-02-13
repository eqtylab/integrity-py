#[cfg(test)]
mod sql_tests {
    use integrity::lineage::models::statements::{
        AssociationStatement, ComputationStatement, DataStatement, MetadataStatement, Statement,
        StatementTrait, StorageStatement,
    };

    use super::super::Sqlite;
    use crate::indexer::Graph;

    async fn setup_db() -> Sqlite {
        let db = Sqlite::new("sqlite::memory:").await.unwrap();
        db.init().await.unwrap();
        db
    }

    fn make_graph(id: uuid::Uuid, name: &str) -> Graph {
        Graph {
            id,
            name: name.to_string(),
            parent: None,
            statements: None,
        }
    }

    #[tokio::test]
    async fn test_register_computation_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000010");
        let graph = make_graph(graph_id, "comp_test");
        db.create_graph(&graph).await.unwrap();

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
        db.register_statement(&comp_statement, &graph_id)
            .await
            .unwrap();

        // Verify by retrieving the graph
        let statements = db.retrieve_statements(&graph_id).await.unwrap();
        assert!(statements.iter().any(|s| s.get_id() == statement_id));
    }

    #[tokio::test]
    async fn test_register_data_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000011");
        let graph = make_graph(graph_id, "data_test");
        db.create_graph(&graph).await.unwrap();

        let did = String::from("did:key:data_statement");
        let data_cid = String::from("urn:cid:input1");
        let statement = DataStatement::create(vec![data_cid.clone()], did.clone(), None)
            .await
            .unwrap();
        let data_statement_id = statement.get_id();

        let data_statement = Statement::DataRegistration(statement);
        db.register_statement(&data_statement, &graph_id)
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
            &graph_id,
        )
        .await
        .unwrap();

        // Verify by retrieving the graph - should contain both computation and data statements
        let statements = db.retrieve_statements(&graph_id).await.unwrap();
        assert!(statements.iter().any(|s| s.get_id() == data_statement_id));
    }

    #[tokio::test]
    async fn test_register_metadata_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000012");
        let graph = make_graph(graph_id, "metadata_test");
        db.create_graph(&graph).await.unwrap();

        let did = String::from("did:key:metadata_statement");
        let subject = String::from("urn:cid:metadata1");
        let metadata = String::from("the metadata");
        let statement =
            MetadataStatement::create(subject.clone(), metadata.clone(), did.clone(), None)
                .await
                .unwrap();
        let metadata_statement_id = statement.get_id();

        let meta_statement = Statement::MetadataRegistration(statement);
        db.register_statement(&meta_statement, &graph_id)
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
            &graph_id,
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let statements = db.retrieve_statements(&graph_id).await.unwrap();
        assert!(statements
            .iter()
            .any(|s| s.get_id() == metadata_statement_id));
    }

    #[tokio::test]
    async fn test_register_storage_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000013");
        let graph = make_graph(graph_id, "storage_test");
        db.create_graph(&graph).await.unwrap();

        let did = String::from("did:key:storage_statement");
        let subject = String::from("urn:cid:storage");
        let stored_on = String::from("urn:cid:stored_on");
        let statement =
            StorageStatement::create(subject.clone(), stored_on.clone(), None, did.clone(), None)
                .await
                .unwrap();
        let storage_statement_id = statement.get_id();

        let storage_statement = Statement::StorageRegistration(statement);
        db.register_statement(&storage_statement, &graph_id)
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
            &graph_id,
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let statements = db.retrieve_statements(&graph_id).await.unwrap();
        assert!(statements
            .iter()
            .any(|s| s.get_id() == storage_statement_id));
    }

    #[tokio::test]
    async fn test_register_association_statement() {
        let db = setup_db().await;
        let graph_id = uuid::uuid!("00000000-0000-0000-0000-000000000014");
        let graph = make_graph(graph_id, "association_test");
        db.create_graph(&graph).await.unwrap();

        let did = String::from("did:key:association_statement");
        let subject = String::from("urn:cid:association_subjectx");
        let association = String::from("urn:cid:association_associate");
        let statement =
            AssociationStatement::create(subject.clone(), association.clone(), did.clone(), None)
                .await
                .unwrap();
        let association_statement_id = statement.get_id();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, &graph_id)
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
            &graph_id,
        )
        .await
        .unwrap();

        // Verify by retrieving the graph
        let statements = db.retrieve_statements(&graph_id).await.unwrap();
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
        let uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");
        db.register_statement(&assoc_statement, &uuid)
            .await
            .unwrap();

        let associations = db.get_associations_for_subject(&subject).await.unwrap();
        assert_eq!(associations.len(), 1);
        assert_eq!(associations.first(), Some(&associate1));

        let associate2 = String::from("urn:cid:association_second");
        let statement =
            AssociationStatement::create(subject.clone(), associate2.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, &uuid)
            .await
            .unwrap();
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
        let uuid = &uuid::uuid!("00000000-0000-0000-1000-500000000000");
        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, uuid).await.unwrap();

        let subjects = db.get_subjects_for_association(&associate).await.unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects.first(), Some(&subject1));

        let subject2 = String::from("urn:cid:association_subject2");
        let statement =
            AssociationStatement::create(subject2.clone(), associate.clone(), did.clone(), None)
                .await
                .unwrap();

        let assoc_statement = Statement::AssociationRegistration(statement);
        db.register_statement(&assoc_statement, uuid).await.unwrap();
        let subjects = db.get_subjects_for_association(&associate).await.unwrap();
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[0], subject1);
        assert_eq!(subjects[1], subject2);
    }

    #[tokio::test]
    async fn test_statement_retrieval_with_hierarchy() {
        let db = setup_db().await;
        let root_graph_id = uuid::uuid!("00000000-0000-0000-0000-500000000001");
        let graph = make_graph(root_graph_id, "Root Graph");
        db.create_graph(&graph).await.unwrap();

        let child_graph_id = uuid::uuid!("00000000-0000-0000-0000-500000000002");
        let graph = make_graph(child_graph_id, "Child Graph");
        db.create_graph(&graph).await.unwrap();

        let child_graph_id_2 = uuid::uuid!("00000000-0000-0000-0000-500000000003");
        let graph = make_graph(child_graph_id_2, "Child Graph 2");
        db.create_graph(&graph).await.unwrap();

        let input_data = vec![
            "urn:cid:comp_data_input_1".to_owned(),
            "urn:cid:comp_data_input_2".to_owned(),
        ];
        let statement =
            DataStatement::create(input_data.clone(), "did:key:unit_test".to_owned(), None)
                .await
                .unwrap();

        let data_input = Statement::DataRegistration(statement);
        db.register_statement(&data_input, &root_graph_id)
            .await
            .unwrap();

        let output_data = vec!["urn:cid:comp_data_output".to_owned()];
        let statement =
            DataStatement::create(output_data.clone(), "did:key:unit_test".to_owned(), None)
                .await
                .unwrap();

        let data_output = Statement::DataRegistration(statement);
        db.register_statement(&data_output, &child_graph_id)
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
        db.register_statement(&metadata, &child_graph_id_2)
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
        db.register_statement(&comp_statement, &child_graph_id)
            .await
            .unwrap();

        // Check that the statements in the parent graphs get pulled in
        let statements = db.retrieve_statements(&child_graph_id).await.unwrap();
        assert_eq!(statements.len(), 3);

        // Register the same statement in a lower child project
        db.register_statement(&comp_statement, &child_graph_id_2)
            .await
            .unwrap();

        // Check that the statements in the parent graphs get pulled in from a lower child
        let statements = db.retrieve_statements(&child_graph_id_2).await.unwrap();
        assert_eq!(statements.len(), 4);
    }
}
