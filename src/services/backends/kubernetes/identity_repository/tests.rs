use super::*;
use crate::services::backends::kubernetes::common::fixtures::{create_mock_identity_providers, get_kubeconfig};
use boxer_core::testing::create_namespace;
use kube::{Api, Client};
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};
use tokio::time::{sleep, timeout};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)] // Dead code is allowed here because this struct is used in kubernetes
struct KubernetesIdentityRepositoryTest {
    api: Arc<Api<IdentityProvider>>,
    repository: Arc<KubernetesIdentityRepository>,
}

static LABEL_SELECTOR_KEY: &str = "repository.boxer.io/type";
const LABEL_SELECTOR_VALUE: &str = "identity-provider";

impl AsyncTestContext for KubernetesIdentityRepositoryTest {
    async fn setup() -> KubernetesIdentityRepositoryTest {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");
        let client = Client::try_from(config.clone()).expect("Failed to create client");

        let api: Arc<Api<IdentityProvider>> = Arc::new(Api::namespaced(client.clone(), namespace.as_str()));

        create_mock_identity_providers(api.clone(), namespace.clone(), LABEL_SELECTOR_KEY, LABEL_SELECTOR_VALUE).await;

        let config = KubernetesResourceManagerConfig {
            namespace: namespace.clone(),
            label_selector_key: LABEL_SELECTOR_KEY.to_string(),
            label_selector_value: LABEL_SELECTOR_VALUE.to_string(),
            lease_name: "identities".to_string(),
            kubeconfig: config,
            lease_duration: Duration::from_secs(5),
            renew_deadline: Duration::from_secs(3),
            claimant: "boxer".to_string(),
        };
        let repository = KubernetesIdentityRepository::start(config)
            .await
            .expect("Failed to start repository");

        KubernetesIdentityRepositoryTest {
            api,
            repository: Arc::new(repository),
        }
    }

    async fn teardown(self) {
        // do nothing
    }
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-1".to_string();
    let user = "user1".to_string();

    // Act
    let external_identity = ctx
        .repository
        .get((provider.clone(), user.clone()))
        .await
        .expect("Failed to get external identity");

    // Assert
    assert_eq!(external_identity.clone().user_id, "user1");
    assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_not_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-1".to_string();
    let user = "user3".to_string();

    // Act
    let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(external_identity.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_unexisted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-2".to_string();
    let user = "i_do_not_exist".to_string();

    // Act
    let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(external_identity.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-3".to_string();
    let user = "deleted_user".to_string();

    // Act
    let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(external_identity.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_from_empty_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-4".to_string();
    let user = "user1".to_string();

    // Act
    let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(external_identity.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_get_from_not_existed_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-5".to_string();
    let user = "user1".to_string();

    // Act
    let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(external_identity.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_add_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-1".to_string();
    let user = "new_user".to_string();
    let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

    let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
    // Assert that the user does not exist before upsert
    assert_eq!(old_state.ok(), None);

    // Act
    ctx.repository
        .upsert((provider.clone(), user.clone()), external_identity)
        .await
        .expect("Failed to upsert external identity");

    let external_identity = timeout(DEFAULT_TEST_TIMEOUT, async {
        // We use loop here since the upsert operation is asynchronous and we need to wait for the state to be updated
        loop {
            let result = ctx.repository.get((provider.clone(), user.clone())).await;

            if let Ok(external_identity) = result {
                return external_identity;
            } else {
                // Wait for a short period before retrying
                sleep(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .expect("Failed to get external identity after upsert");

    // Assert
    assert_eq!(external_identity.clone().user_id, "new_user");
    assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_add_to_unexisted_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-5".to_string();
    let user = "new_user".to_string();
    let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

    // Act
    let result = ctx
        .repository
        .upsert((provider.clone(), user.clone()), external_identity)
        .await;

    // Assert
    let message = result.err().unwrap().to_string();
    assert!(
        message.contains("Identity provider \"identity-provider-5\" not found in namespace"),
        "Unexpected error message: {}",
        message
    );
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_add_duplicate(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-1".to_string();
    let user = "user1".to_string();
    let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

    let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
    // Assert that the user does not exist before upsert
    assert_eq!(
        old_state.ok(),
        Some(ExternalIdentity::new(provider.clone(), user.clone()))
    );

    // Act
    ctx.repository
        .upsert((provider.clone(), user.clone()), external_identity)
        .await
        .expect("Failed to upsert external identity");

    // Assert
    let external_identity = ctx
        .repository
        .get((provider.clone(), user.clone()))
        .await
        .expect("Failed to get external identity");

    assert_eq!(external_identity.clone().user_id, "user1");
    assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_add_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-3".to_string();
    let user = "deleted_user".to_string();
    let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

    let old_state = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert that the user does not exist before upsert
    assert_eq!(old_state.ok(), None);

    // Act
    let result = ctx
        .repository
        .upsert((provider.clone(), user.clone()), external_identity)
        .await;

    let message = result.err().unwrap().to_string();
    // Assert
    assert!(
        message.contains("User \"deleted_user\" is inactive in provider \"identity-provider-3\""),
        "Unexpected error message: {}",
        message
    );
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_delete_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-1".to_string();
    let user = "user1".to_string();

    let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
    // Assert that the user exists before delete
    assert_eq!(
        old_state.ok(),
        Some(ExternalIdentity::new(provider.clone(), user.clone()))
    );

    // Act
    ctx.repository
        .delete((provider.clone(), user.clone()))
        .await
        .expect("Failed to delete external identity");

    let new_state = timeout(DEFAULT_TEST_TIMEOUT, async {
        // We use loop here since the upsert operation is asynchronous and we need to wait for the state to be updated
        loop {
            let result = ctx.repository.get((provider.clone(), user.clone())).await;

            if let Ok(_) = result {
                sleep(Duration::from_millis(100)).await;
            } else {
                return result;
            }
        }
    })
    .await
    .expect("Failed to get external identity after upsert");

    // Assert
    assert_eq!(new_state.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_delete_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-3".to_string();
    let user = "deleted_user".to_string();

    // Act
    ctx.repository
        .delete((provider.clone(), user.clone()))
        .await
        .expect("Failed to delete external identity");

    let new_state = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(new_state.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_delete_unexisted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-4".to_string();
    let user = "i_do_not_exist".to_string();

    // Act
    ctx.repository
        .delete((provider.clone(), user.clone()))
        .await
        .expect("Failed to delete external identity");

    let new_state = ctx.repository.get((provider.clone(), user.clone())).await;

    // Assert
    assert_eq!(new_state.ok(), None);
}

#[test_context(KubernetesIdentityRepositoryTest)]
#[tokio::test]
async fn test_delete_from_unexisted_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
    // Arrange
    let provider = "identity-provider-5".to_string();
    let user = "user1".to_string();

    // Act
    let result = ctx.repository.delete((provider.clone(), user.clone())).await;

    // Assert
    let message = result.err().unwrap().to_string();
    assert!(
        message.contains("Identity provider \"identity-provider-5\" not found in namespace:"),
        "Unexpected error message: {}",
        message
    );
}
