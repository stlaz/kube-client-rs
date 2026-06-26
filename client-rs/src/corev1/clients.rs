use k8s_openapi::api::core::v1::{ConfigMap, Namespace, Pod, Secret, Service};

use crate::gentype::ResourceClientWithList;
use crate::rest::RestClient;

pub struct CoreV1Client<'c> {
    rest_client: &'c RestClient,
}

impl<'c> CoreV1Client<'c> {
    pub fn new(rest_client: &'c RestClient) -> Self {
        CoreV1Client { rest_client }
    }

    pub fn rest_client(&self) -> &RestClient {
        self.rest_client
    }

    pub fn namespaces(&self) -> NamespaceClient<'_> {
        new_namespace_client(self.rest_client)
    }
    pub fn pods(&self, namespace: &str) -> PodClient<'_> {
        new_pod_client(self.rest_client, namespace)
    }
    pub fn services(&self, namespace: &str) -> ServiceClient<'_> {
        new_service_client(self.rest_client, namespace)
    }
    pub fn configmaps(&self, namespace: &str) -> ConfigMapClient<'_> {
        new_configmap_client(self.rest_client, namespace)
    }
    pub fn secrets(&self, namespace: &str) -> SecretClient<'_> {
        new_secret_client(self.rest_client, namespace)
    }
}

pub type NamespaceClient<'c> = ResourceClientWithList<'c, Namespace>;

fn new_namespace_client<'c>(client: &'c RestClient) -> ResourceClientWithList<'c, Namespace> {
    ResourceClientWithList::new(client, "namespaces".to_string(), None)
}

pub type PodClient<'c> = ResourceClientWithList<'c, Pod>;

fn new_pod_client<'c>(client: &'c RestClient, namespace: &str) -> ResourceClientWithList<'c, Pod> {
    ResourceClientWithList::new(client, "pods".to_string(), Some(namespace.to_string()))
}

pub type ServiceClient<'c> = ResourceClientWithList<'c, Service>;

fn new_service_client<'c>(client: &'c RestClient, namespace: &str) -> ServiceClient<'c> {
    ResourceClientWithList::new(client, "services".to_string(), Some(namespace.to_string()))
}

pub type ConfigMapClient<'c> = ResourceClientWithList<'c, ConfigMap>;

fn new_configmap_client<'c>(client: &'c RestClient, namespace: &str) -> ConfigMapClient<'c> {
    ResourceClientWithList::new(
        client,
        "configmaps".to_string(),
        Some(namespace.to_string()),
    )
}

pub type SecretClient<'c> = ResourceClientWithList<'c, Secret>;

fn new_secret_client<'c>(client: &'c RestClient, namespace: &str) -> SecretClient<'c> {
    ResourceClientWithList::new(client, "secrets".to_string(), Some(namespace.to_string()))
}
