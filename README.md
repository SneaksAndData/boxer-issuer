# Introduction  
Boxer Authorization (AuthZ) API with Signature Based Authentication Provider (Authn)


# Usage

## Enabling Boxer Authorization (AuthZ) or Authentication (AuthN) in your app
* C#: Use [esd-services-api-client-dotnet](https://github.com/SneaksAndData/esd-services-api-client-dotnet)
* Python: Use [esd-services-api-client](https://github.com/SneaksAndData/esd-services-api-client)


# Integration testing
To run integration tests, you need to set up a test environment with the following:

```shell
$ docker-compose up -d
```


To obtain an external token for testing, you can use the following command:

```shell
$ curl \
  -d "client_id=test_client" \
  -d "client_secret=test_client_secret" \
  -d "username=test_user" \
  -d "password=test_user_password" \
  -d "grant_type=password" \
  "http://localhost:8080/realms/master/protocol/openid-connect/token"
```
