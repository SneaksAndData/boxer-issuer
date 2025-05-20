mod util;

use std::sync::RwLock;
use boxer_issuer::models::api::external::identity::ExternalIdentity;
use boxer_issuer::models::principal::Principal;
use boxer_issuer::services::base::upsert_repository::{PrincipalAssociationRepository, PrincipalRepository, SchemaRepository};
use cedar_policy::{Entity, SchemaFragment};
use std::collections::HashMap;
use std::sync::Arc;
use cedar_policy::ffi::validate;
use boxer_issuer::services::identity_validator_provider;
use boxer_issuer::services::identity_validator_provider::ExternalIdentityValidationService;
use boxer_issuer::services::token_service::TokenService;
use crate::util::validators::AlwaysValid;

#[tokio::test]
async fn it_adds_two() {
    let schemas_repository: Arc<SchemaRepository> = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let entities_repository: Arc<PrincipalRepository> = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let principal_association_repository: Arc<PrincipalAssociationRepository> = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    
    let schema_name = "schema".to_string();
    let schema_fragment = SchemaFragment::from_json_str(SCHEMA).unwrap();
    schemas_repository.upsert(schema_name.clone(), schema_fragment.clone()).await.unwrap();
    
    let schema = schema_fragment.try_into().unwrap();
    let entity = Entity::from_json_str(USER, Some(&schema)).unwrap();
    let key = ("User".to_string(), "Alice".to_string());
    let principal = Principal::new(entity.clone(), schema_name.clone());
    entities_repository.upsert(key.clone(), principal.clone()).await.unwrap();
    
    let ext_id = ("identity_provider".to_string(), "user_id".to_string());
    let external_identity = ExternalIdentity::from(ext_id);
    
    principal_association_repository.upsert(external_identity, (key.0, schema_name)).await.unwrap();

    let validator= Arc::new(AlwaysValid { identity: external_identity.clone() });
    let validator_service = identity_validator_provider::new();
    validator_service.put(external_identity.clone(), validator).await.unwrap();
    
    let token_service = TokenService::new(
        validator_provider,
        schemas_repository.clone(),
        entities_repository.clone());
    
    let token = token_service.issue_token(
        external_identity.clone(),
        principal_association_repository.clone(),
    ).await.unwrap();
    assert_eq!(true, false);
}

const SCHEMA: &str = r#"
{
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
            }
        },
        "actions": {}
    }
}
"#;
    
const USER: &str = r#"
{
        "uid": {
            "type": "PhotoApp::User",
            "id": "alice"
        },
        "attrs": {
            "userId": "897345789237492878",
            "personInformation": {
                "age": 25,
                "name": "alice"
            }
        },
        "parents": [
            {
                "type": "PhotoApp::UserGroup",
                "id": "alice_friends"
            },
            {
                "type": "PhotoApp::UserGroup",
                "id": "AVTeam"
            }
        ]
}
"#;