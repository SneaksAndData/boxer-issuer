use super::*;
use crate::services::backends::kubernetes::common::RepositoryConfig;
use crate::services::base::upsert_repository::UpsertRepository;
use k8s_openapi::api::core::v1::{ConfigMap, Namespace};
use kube::api::PostParams;
use kube::{Api, Client};
use serde_json::json;
use std::println as info;
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};
use uuid::Uuid;

#[allow(dead_code)]
const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)] // Dead code is allowed here because this struct is used in kubernetes
struct KubernetesSchemaRepositoryTest {
    api: Arc<Api<ConfigMap>>,
    repository: Arc<KubernetesSchemaRepository>,
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

        let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace.as_str());

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
            api: Arc::new(api),
            repository: Arc::new(repository),
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
    let schema_str = r#"{
    "PhotoApp": {
        "commonTypes": {
            "PersonType": {
                "type": "Record",
                "attributes": {
                    "age": {
                        "type": "Long"
                    },
                    "name": {
                        "type": "String"
                    }
                }
            },
            "ContextType": {
                "type": "Record",
                "attributes": {
                    "ip": {
                        "type": "Extension",
                        "name": "ipaddr",
                        "required": false
                    },
                    "authenticated": {
                        "type": "Boolean",
                        "required": true
                    }
                }
            }
        },
        "entityTypes": {
            "User": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "userId": {
                            "type": "String"
                        },
                        "personInformation": {
                            "type": "PersonType"
                        }
                    }
                },
                "memberOfTypes": [
                    "UserGroup"
                ]
            },
            "UserGroup": {
                "shape": {
                    "type": "Record",
                    "attributes": {}
                }
            },
            "Photo": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "account": {
                            "type": "Entity",
                            "name": "Account",
                            "required": true
                        },
                        "private": {
                            "type": "Boolean",
                            "required": true
                        }
                    }
                },
                "memberOfTypes": [
                    "Album",
                    "Account"
                ]
            },
            "Album": {
                "shape": {
                    "type": "Record",
                    "attributes": {}
                }
            },
            "Account": {
                "shape": {
                    "type": "Record",
                    "attributes": {}
                }
            }
        },
        "actions": {
            "viewPhoto": {
                "appliesTo": {
                    "principalTypes": [
                        "User",
                        "UserGroup"
                    ],
                    "resourceTypes": [
                        "Photo"
                    ],
                    "context": {
                        "type": "ContextType"
                    }
                }
            },
            "createPhoto": {
                "appliesTo": {
                    "principalTypes": [
                        "User",
                        "UserGroup"
                    ],
                    "resourceTypes": [
                        "Photo"
                    ],
                    "context": {
                        "type": "ContextType"
                    }
                }
            },
            "listPhotos": {
                "appliesTo": {
                    "principalTypes": [
                        "User",
                        "UserGroup"
                    ],
                    "resourceTypes": [
                        "Photo"
                    ],
                    "context": {
                        "type": "ContextType"
                    }
                }
            }
        }
    }
}"#;
    let schema_fragment = SchemaFragment::from_json_str(schema_str).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    let retrieved_schema = ctx.api.get(&name).await.expect("Failed to get schema from Kubernetes");

    // Assert
    assert_eq!(retrieved_schema.metadata.name.unwrap(), "test-schema");
}
