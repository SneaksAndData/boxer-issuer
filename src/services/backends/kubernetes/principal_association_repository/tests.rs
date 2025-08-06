use super::*;
use crate::services::backends::kubernetes::common::fixtures::create_mock_identity_providers;
use crate::services::backends::kubernetes::identity_repository::KubernetesIdentityRepository;
use boxer_core::services::base::upsert_repository::CanDelete;
use k8s_openapi::api::core::v1::Namespace;
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
struct KubernetesPrincipalAssociationRepositoryTest {
    api: Arc<Api<IdentityProvider>>,
    repository: Arc<KubernetesPrincipalAssociationRepository>,
    identity_repository: Arc<KubernetesIdentityRepository>,
}

static LABEL_SELECTOR_KEY: &str = "repository.boxer.io/type";
const LABEL_SELECTOR_VALUE: &str = "principal";

impl AsyncTestContext for KubernetesPrincipalAssociationRepositoryTest {
    async fn setup() -> KubernetesPrincipalAssociationRepositoryTest {
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

        let api: Arc<Api<IdentityProvider>> = Arc::new(Api::namespaced(client.clone(), namespace.as_str()));
        create_mock_identity_providers(api.clone(), namespace.clone(), LABEL_SELECTOR_KEY, LABEL_SELECTOR_VALUE).await;

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

        let repository = KubernetesPrincipalAssociationRepository::start(config.clone())
            .await
            .expect("Failed to start repository");

        let identity_repository = KubernetesIdentityRepository::start(config)
            .await
            .expect("Failed to start repository");

        KubernetesPrincipalAssociationRepositoryTest {
            api,
            repository: Arc::new(repository),
            identity_repository: Arc::new(identity_repository),
        }
    }

    async fn teardown(self) {
        // do nothing
    }
}

#[test_context(KubernetesPrincipalAssociationRepositoryTest)]
#[tokio::test]
async fn test_create_association(ctx: &mut KubernetesPrincipalAssociationRepositoryTest) {
    // Arrange
    let external_identity = ExternalIdentity::new("identity-provider-1".to_string(), "user1".to_string());
    let principal_identity = PrincipalIdentity::new(
        "test-schema-entities".to_string(),
        "PhotoApp::User::\"alice\"".to_string(),
    );

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    ctx.repository
        .get(external_identity.clone())
        .await
        .expect_err("Principal should not exist before upsert");

    // Act
    ctx.repository
        .upsert(external_identity.clone(), principal_identity)
        .await
        .expect("Failed to upsert principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    let association = ctx
        .repository
        .get(external_identity.clone())
        .await
        .expect("Principal should not exist before upsert");

    // Assert
    assert_eq!(
        association.schema_id(),
        "test-schema-entities",
        "Schema ID should match the upserted principal identity"
    );
}

#[test_context(KubernetesPrincipalAssociationRepositoryTest)]
#[tokio::test]
async fn test_delete_principal(ctx: &mut KubernetesPrincipalAssociationRepositoryTest) {
    // Arrange
    let external_identity = ExternalIdentity::new("identity-provider-1".to_string(), "user1".to_string());
    let principal_identity = PrincipalIdentity::new(
        "test-schema-entities".to_string(),
        "PhotoApp::User::\"alice\"".to_string(),
    );

    ctx.repository
        .upsert(external_identity.clone(), principal_identity)
        .await
        .expect("Failed to upsert principal");

    ctx.identity_repository
        .delete((
            external_identity.identity_provider.clone(),
            external_identity.user_id.clone(),
        ))
        .await
        .expect("Failed to delete identity provider");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Act
    // Assert
    let principal_result = ctx.repository.get(external_identity.clone()).await;
    assert!(principal_result.is_err(), "Principal should not exist after deletion");
}

#[test_context(KubernetesPrincipalAssociationRepositoryTest)]
#[tokio::test]
async fn test_update_schema(ctx: &mut KubernetesPrincipalAssociationRepositoryTest) {
    // Arrange
    let external_identity = ExternalIdentity::new("identity-provider-1".to_string(), "user1".to_string());
    let principal_identity = PrincipalIdentity::new(
        "test-schema-entities".to_string(),
        "PhotoApp::User::\"alice\"".to_string(),
    );
    let new_principal_identity = PrincipalIdentity::new(
        "test-schema-entities".to_string(),
        "PhotoApp::User::\"bob\"".to_string(),
    );
    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    ctx.repository
        .upsert(external_identity.clone(), principal_identity.clone())
        .await
        .expect("Failed to upsert principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Act
    ctx.repository
        .upsert(external_identity.clone(), new_principal_identity.clone())
        .await
        .expect("Failed to update principal");

    sleep(Duration::from_secs(1)).await; // Ensure the schema is created before retrieving it

    // Assert
    let principal_result = ctx
        .repository
        .get(external_identity.clone())
        .await
        .expect("Failed to get schema after deletion");

    assert_eq!(new_principal_identity, principal_result, "Age should be updated");
}
