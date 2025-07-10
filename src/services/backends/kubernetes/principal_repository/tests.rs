use super::*;
use crate::services::backends::kubernetes::principal_repository::test_principal::{principal, updated_principal};
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
struct KubernetesPrincipalRepositoryTest {
    raw_api: Arc<Api<ConfigMap>>,
    data_api: Arc<Api<PrincipalConfigMap>>,
    repository: Arc<KubernetesPrincipalRepository>,
}

static LABEL_SELECTOR_KEY: &str = "repository.boxer.io/type";
const LABEL_SELECTOR_VALUE: &str = "principal";

impl AsyncTestContext for KubernetesPrincipalRepositoryTest {
    async fn setup() -> KubernetesPrincipalRepositoryTest {
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
        let data_api: Api<PrincipalConfigMap> = Api::namespaced(client.clone(), namespace.as_str());

        let config = KubernetesResourceManagerConfig {
            namespace: namespace.clone(),
            label_selector_key: LABEL_SELECTOR_KEY.to_string(),
            label_selector_value: LABEL_SELECTOR_VALUE.to_string(),
            lease_name: "principals".to_string(),
            kubeconfig: config,
            lease_duration: Duration::from_secs(5),
            renew_deadline: Duration::from_secs(3),
            claimant: "boxer".to_string(),
        };

        let repository = KubernetesPrincipalRepository::start(config)
            .await
            .expect("Failed to start repository");

        let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace.as_str());
        let data = btreemap! {
            "active".to_string() => "[]".to_string(),
            "inactive".to_string() => "[]".to_string(),
        };
        let config_map = ConfigMap {
            data: Some(data),
            metadata: ObjectMeta {
                name: Some("test-schema-entities".to_string()),
                namespace: Some(namespace.clone()),
                labels: Some(btreemap! {
                    LABEL_SELECTOR_KEY.to_string() => LABEL_SELECTOR_VALUE.to_string(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        api.create(&PostParams::default(), &config_map)
            .await
            .expect("Failed to create ConfigMap");

        KubernetesPrincipalRepositoryTest {
            raw_api: Arc::new(raw_api),
            data_api: Arc::new(data_api),
            repository: Arc::new(repository),
        }
    }

    async fn teardown(self) {
        // do nothing
    }
}

#[test_context(KubernetesPrincipalRepositoryTest)]
#[tokio::test]
async fn test_create_principal(ctx: &mut KubernetesPrincipalRepositoryTest) {
    // Arrange
    let name = "test-schema-entities";
    let principal_id = PrincipalIdentity::new(name.to_string(), "PhotoApp::User::\"Alice\"".to_string());

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Act
    ctx.repository
        .upsert(principal_id, principal(name.to_string()))
        .await
        .expect("Failed to upsert principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    let retrieved_principal = ctx
        .raw_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");

    // Assert
    assert_eq!(retrieved_principal.metadata.name.unwrap(), "test-schema-entities");
}

#[test_context(KubernetesPrincipalRepositoryTest)]
#[tokio::test]
async fn test_delete_principal(ctx: &mut KubernetesPrincipalRepositoryTest) {
    // Arrange
    let name = "test-schema-entities";
    let principal_id = PrincipalIdentity::new(name.to_string(), "PhotoApp::User::\"alice\"".to_string());
    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    ctx.repository
        .upsert(principal_id.clone(), principal(name.to_string()))
        .await
        .expect("Failed to upsert principal");

    let retrieved_principal = ctx
        .raw_api
        .get(&name)
        .await
        .expect("Failed to get schema from Kubernetes");
    assert_eq!(retrieved_principal.metadata.name.unwrap(), "test-schema-entities");

    // Act
    ctx.repository
        .delete(principal_id.clone())
        .await
        .expect("Failed to delete principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Assert
    let principal_result = ctx.repository.get(principal_id).await;
    assert!(principal_result.is_err(), "Principal should not exist after deletion");
}
//
#[test_context(KubernetesPrincipalRepositoryTest)]
#[tokio::test]
async fn test_update_schema(ctx: &mut KubernetesPrincipalRepositoryTest) {
    // Arrange
    let name = "test-schema-entities";
    let principal_id = PrincipalIdentity::new(name.to_string(), "PhotoApp::User::\"alice\"".to_string());
    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    ctx.repository
        .upsert(principal_id.clone(), principal(name.to_string()))
        .await
        .expect("Failed to upsert principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Act
    ctx.repository
        .upsert(principal_id.clone(), updated_principal(name.to_string()))
        .await
        .expect("Failed to update principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Assert
    let principal_result = ctx
        .repository
        .get(principal_id)
        .await
        .expect("Failed to get schema after deletion");

    let entity = principal_result.get_entity();
    let old_principal = principal(name.to_string());
    let old_entity = old_principal.get_entity();

    assert_ne!(entity.attr("age"), old_entity.attr("age"), "Age should be updated");
}
