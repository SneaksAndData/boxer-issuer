# Setting up the test environment

## How to start dependencies

The default test environment for the boxer requires Docker and Docker Compose.
To launch the test environment, run the following command:

```shell
$ docker-compose up -d
```

## How to setup boxer for testing

To set up the boxer entities, you can do the following steps:

### Register an identity provider

Create an identity

```shell
curl -X POST 'http://localhost:8888/identity_provider/oidc/provider' \
--header 'Content-Type: application/json' \
--data '{
  "user_id_claim": "preferred_username",
  "discovery_url": "http://localhost:8080/realms/master/",
  "issuers": [ "http://localhost:8080/realms/master" ],
  "audiences": [ "account" ]
}'
```

### Register an identity

Create an identity

```shell
curl -X POST 'http://localhost:8888/identity/provider/test_user'
```

### Create a schema

```shell
SCHEMA_DOCUMENT=$(cat << 'EOF'
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
EOF
)
curl -X POST 'http://localhost:8888/schema/test' \
--header 'Content-Type: application/json' \
--data $SCHEMA_DOCUMENT
```

### Validate that the schema is created

```shell
$ curl -X GET 'http://localhost:8888/schema/test'
```

### Create a principal

```shell

PRINCIPAL=$(cat << 'EOF'
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
EOF
)

curl -X POST 'http://localhost:8888/principal/test' \
--header 'Content-Type: application/json' \
--data $PRINCIPAL

```

### Create a principal association

```shell
curl -X POST 'http://localhost:8888/association/' \
--header 'Content-Type: application/json' \
--data '{
    "identity_provider": "provider",
    "identity": "test_user",
    "principal_schema": "test",
    "principal_id": "User::\"alice\""
}'
```

## Validate the setup

### Getting the external token

```shell
export EXTERNAL_TOKEN=$(curl \
  -d "client_id=test_client" \
  -d "client_secret=test_client_secret" \
  -d "username=test_user" \
  -d "password=test_user_password" \
  -d "grant_type=password" \
  "http://localhost:8080/realms/master/protocol/openid-connect/token" | jq -r '.access_token')
```

### Getting the boxer token

```shell
curl -X GET 'http://localhost:8888/token/provider' --header "Authorization: Bearer $EXTERNAL_TOKEN"
```
