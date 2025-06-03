use std::collections::HashMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Client;

type ExternalIdentities = HashMap<String, Vec<String>>;
struct IdentitiesConfigMap {
    metadata: ObjectMeta,
    data: ExternalIdentities,
}

struct KubernetesIdentityRepository {
    current_version: String,
    external_identities: ExternalIdentities,
}

pub  struct KubernetesBackend {
    client: Client
}