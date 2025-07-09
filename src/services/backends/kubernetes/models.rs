use crate::services::backends::kubernetes::models::base::WithMetadata;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Resource;
use std::collections::BTreeMap;

/// Common traits and functions for Kubernetes resources
pub mod base;

/// Creates an empty resource with the specified metadata.
pub fn empty<K>(name: String, namespace: String, labels: BTreeMap<String, String>) -> K
where
    K: Resource + Default + WithMetadata<ObjectMeta>,
{
    let metadata = empty_metadata(name, namespace, labels);
    K::default().with_metadata(metadata)
}

/// Creates an empty `ObjectMeta` with the specified name, namespace, and labels.
pub fn empty_metadata(name: String, namespace: String, labels: BTreeMap<String, String>) -> ObjectMeta {
    ObjectMeta {
        name: Some(name),
        namespace: Some(namespace),
        labels: Some(labels),
        ..Default::default()
    }
}
