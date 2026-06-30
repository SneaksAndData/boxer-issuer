terraform {
  required_providers {
    keycloak = {
      source  = "keycloak/keycloak"
      version = "= 5.0.0"
    }
  }
}

provider "keycloak" {
  client_id = "admin-cli"
  username  = "admin"
  password  = "admin"
  url       = "http://localhost:5555/auth"
}

data "keycloak_realm" "master" {
  realm = "master"
}

resource "keycloak_openid_client" "test_client" {
  name                         = "test_client"
  access_type                  = "PUBLIC"
  client_id                    = "test_client"
  client_secret                = "test_client_secret"
  realm_id                     = data.keycloak_realm.master.id
  enabled                      = true
  direct_access_grants_enabled = true
}

resource "keycloak_openid_client_default_scopes" "client_default_scopes" {
  realm_id  = data.keycloak_realm.master.id
  client_id = keycloak_openid_client.test_client.id

  default_scopes = [
    "profile",
    "email",
    "roles",
    "web-origins",
  ]
}

resource "keycloak_user" "test_user" {
  realm_id = data.keycloak_realm.master.id
  username = "test_user"
  enabled  = true

  initial_password {
    value = "test-user-password"
  }
}

resource "keycloak_user" "test_root" {
  realm_id = data.keycloak_realm.master.id
  username = "test_root"
  enabled  = true

  initial_password {
    value = "test-root-password"
  }
}

