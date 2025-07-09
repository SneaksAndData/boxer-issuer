use crate::models::principal::Principal;
use cedar_policy::Entity;
use serde_json::json;

pub fn principal(schema_name: String) -> Principal {
    let entity = json!({
        "uid": {
            "type": "PhotoApp::User",
            "id": "alice"
        },
        "attrs": {
            "userId": "8",
            "age": 25,
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
    });

    Principal::new(Entity::from_json_value(entity, None).unwrap(), schema_name)
}

pub fn updated_principal(schema_name: String) -> Principal {
    let entity = json!({
        "uid": {
            "type": "PhotoApp::User",
            "id": "alice"
        },
        "attrs": {
            "userId": "9",
            "age": 35,
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
    });

    Principal::new(Entity::from_json_value(entity, None).unwrap(), schema_name)
}
