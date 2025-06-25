use super::*;
use crate::services::backends::kubernetes::common::RepositoryConfig;
use crate::services::backends::kubernetes::schema_repository::test_reduced_schema::reduced_schema;
use crate::services::backends::kubernetes::schema_repository::test_schema::schema;
use crate::services::base::upsert_repository::UpsertRepository;
use cedar_policy::Schema;
use k8s_openapi::api::core::v1::{ConfigMap, Namespace};
use kube::api::PostParams;
use kube::{Api, Client};
use serde_json::json;
use std::println as info;
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};
use tokio::time::sleep;
use uuid::Uuid;

#[allow(dead_code)]
const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)] // Dead code is allowed here because this struct is used in kubernetes
struct KubernetesSchemaRepositoryTest {
    raw_api: Arc<Api<ConfigMap>>,
    data_api: Arc<Api<SchemaConfigMap>>,
    repository: Arc<KubernetesSchemaRepository>,
    schema_str: String,
}

static LABEL_SELECTOR_KEY: &str = "repository.boxer.io/type";
const LABEL_SELECTOR_VALUE: &str = "schema";

impl AsyncTestContext for KubernetesSchemaRepositoryTest {
    async fn setup() -> KubernetesSchemaRepositoryTest {
        let config = super::super::common::fixtures::get_kubeconfig()
            .await
            .expect("Failed to get kubeconfig");

        let client = Client::try_from(config.clone()).expect("Failed to create Kubernetes client");
        let namespace = Uuid::new_v4().to_string();
        info!("Using namespace: {}", namespace);

        let namespaces: Api<Namespace> = Api::all(client.clone());
        let namespace_json = json!({ "metadata": { "name": namespace.clone() } });
        let ns = serde_json::from_value(namespace_json).expect("Failed to deserialize namespace");

        namespaces
            .create(&PostParams::default(), &ns)
            .await
            .expect("Create Namespace failed");

        let raw_api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace.as_str());
        let data_api: Api<SchemaConfigMap> = Api::namespaced(client.clone(), namespace.as_str());

        let config = RepositoryConfig {
            namespace: namespace.clone(),
            label_selector_key: LABEL_SELECTOR_KEY.to_string(),
            label_selector_value: LABEL_SELECTOR_VALUE.to_string(),
            kubeconfig: config,
        };

        let repository = KubernetesSchemaRepository::start(config)
            .await
            .expect("Failed to start repository");

        KubernetesSchemaRepositoryTest {
            raw_api: Arc::new(raw_api),
            data_api: Arc::new(data_api),
            repository: Arc::new(repository),
            schema_str: serde_json::to_string(&schema()).expect("Failed to serialize schema to JSON"),
        }
    }

    async fn teardown(self) {
        // do nothing
    }
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_create_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    let retrieved_schema = ctx
        .raw_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");

    // Assert
    assert_eq!(retrieved_schema.metadata.name.unwrap(), "test-schema");
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_delete_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    let retrieved_schema = ctx
        .raw_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");
    assert_eq!(retrieved_schema.metadata.name.unwrap(), "test-schema");

    // Act
    ctx.repository
        .delete(name.to_string())
        .await
        .expect("Failed to delete schema");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Assert
    let schema_result = ctx.repository.get(name.to_string()).await;
    let data = ctx
        .data_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");
    assert_eq!(data.metadata.name.unwrap(), "test-schema");
    assert_eq!(data.data.active, "false");
    assert!(schema_result.is_err(), "Schema should not exist after deletion");
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_update_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it
    let retrieved_schema: Schema = ctx
        .repository
        .get(name.to_string())
        .await
        .expect("Failed to get schema from Kubernetes")
        .try_into()
        .expect("Failed to convert schema to Schema type");

    let new_schema_str = serde_json::to_string(&reduced_schema()).expect("Failed to serialize reduced schema to JSON");
    let new_schema_fragment = SchemaFragment::from_json_str(&new_schema_str).expect("Failed to create schema fragment");
    assert_eq!(retrieved_schema.actions().count(), 1);

    // Act
    ctx.repository
        .upsert(name.to_string(), new_schema_fragment)
        .await
        .expect("Failed to update schema");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Assert
    let schema_result: Schema = ctx
        .repository
        .get(name.to_string())
        .await
        .expect("Failed to get schema after deletion")
        .try_into()
        .expect("Failed to convert schema to Schema type");

    let data = ctx
        .data_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");
    assert_eq!(data.data.active, "true");
    assert_eq!(schema_result.actions().count(), 0);
}
