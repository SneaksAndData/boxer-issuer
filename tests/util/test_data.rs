use boxer_issuer::models::api::external::identity::ExternalIdentity;
use boxer_issuer::models::api::external::identity_provider::ExternalIdentityProvider;
use boxer_issuer::models::api::external::token::ExternalToken;
use cedar_policy::{Schema, SchemaFragment};

pub fn schema() -> Schema {
    schema_fragment().try_into().unwrap()
}

pub fn schema_name() -> String {
    "schema".to_string()
}

pub fn principal_type() -> String {
    "user".to_string()
}

pub fn user_name() -> String {
    "alice".to_string()
}

pub fn schema_fragment() -> SchemaFragment {
    SchemaFragment::from_json_str(SCHEMA).unwrap()
}

pub fn external_identity_provider() -> ExternalIdentityProvider {
    ExternalIdentityProvider::from("identity_provider".to_string())
}

pub fn external_token() -> ExternalToken {
    ExternalToken::from("token".to_string())
}

pub fn external_identity_raw() -> (String, String) {
    (external_identity_provider().name(), "user_id".to_string())
}

pub fn external_identity() -> ExternalIdentity {
    ExternalIdentity::from((external_identity_provider().name(), "user_id".to_string()))
}

pub const USER: &str = r#"
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
