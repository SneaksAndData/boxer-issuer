# Setting up the test environment

## How to start dependencies
The default test environment for the boxer requires Docker and Docker Compose.
To launch the test environment, run the following command:

```shell
$ docker-compose up -d
```

## How to setup boxer for testing
To set up the boxer entities, you can do the following steps:

### Add a custom policy

Create a policy
```shell
$ curl -X POST 'http://localhost:8888/policy/test-policy' \
--header 'Content-Type: text/plain' \
--data '// Only the owner can access any resource tagged "private"
forbid ( principal, action, resource )
when { resource.tags.contains("private") }    // assumes that resource has "tags"
unless { resource in principal.account };     // assumes that principal has "account"'
```

Validate that the policy is created
```shell
curl -X GET 'http://localhost:8888/policy/test-policy'
```

### Register an identity

Create an identity
```shell
curl -X POST 'http://localhost:8888/identity/provider/test_user'
```

Attach a created policy to a user
```shell
curl -X POST 'http://localhost:8888/attachment/provider/test_user/test-policy'
```
Validate that an identity is created and a policy is attached
```shell
curl -X GET 'http://localhost:8888/attachment/provider/test_user'
```

## Validate the setup

### Getting the external token

```shell
$ export EXTERNAL_TOKEN=$(curl \
  -d "client_id=test_client" \
  -d "client_secret=test_client_secret" \
  -d "username=test_user" \
  -d "password=test_user_password" \
  -d "grant_type=password" \
  "http://localhost:8080/realms/master/protocol/openid-connect/token" | jq -r '.access_token')
```

### Getting the boxer token
```shell
$ curl -X GET 'http://localhost:8888/token/provider' --header "Authorization: Bearer $EXTERNAL_TOKEN"
```
