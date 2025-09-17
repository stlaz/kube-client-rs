use std::ops::Deref;

use k8s_openapi::serde;

use k8s_openapi::List;
use k8s_openapi::api::core::v1::{ConfigMap, Namespace, Pod, Secret, Service};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use apimachinery::watch::ResourceWatcher;

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

pub struct ResourceClient<'c, T> {
    _client_type: std::marker::PhantomData<T>,

    rest_client: &'c RestClient,
    resource_plural: String,
    base_path: String,
}

pub struct ResourceClientWithList<'c, T> {
    _client_type: std::marker::PhantomData<T>,

    resource_client: ResourceClient<'c, T>,
}

impl<'c, T> ResourceClientWithList<'c, T>
where
    T: serde::de::DeserializeOwned
        + k8s_openapi::Metadata<Ty = ObjectMeta>
        + k8s_openapi::ListableResource,
{
    pub fn new(client: &'c RestClient, resource_plural: String, namespace: Option<String>) -> Self {
        Self {
            _client_type: std::marker::PhantomData,
            resource_client: ResourceClient::new(client, resource_plural, namespace),
        }
    }

    pub fn list(&self) -> Result<List<T>, reqwest::Error> {
        let resp = self
            .rest_client
            .get(format!("{}/{}", self.base_path, self.resource_plural))?
            .error_for_status()?;
        Ok(resp.json::<List<T>>()?)
    }
}

// implement Deref to automatically deref method calls to internal ResourceClient
impl<'c, T> Deref for ResourceClientWithList<'c, T> {
    type Target = ResourceClient<'c, T>;
    fn deref(&self) -> &Self::Target {
        &self.resource_client
    }
}

impl<'c, T> ResourceClient<'c, T>
where
    T: serde::de::DeserializeOwned + k8s_openapi::Metadata<Ty = ObjectMeta>,
{
    pub fn new(client: &'c RestClient, resource_plural: String, namespace: Option<String>) -> Self {
        Self {
            _client_type: std::marker::PhantomData,
            rest_client: client,
            resource_plural: resource_plural,
            base_path: match namespace {
                Some(ns) => format!("/api/v1/namespaces/{}", ns),
                None => "/api/v1".to_string(),
            },
        }
    }
    pub fn get(&self, name: &str) -> Result<T, reqwest::Error> {
        let resp = self
            .rest_client
            .get(format!(
                "{}/{}/{}",
                self.base_path, self.resource_plural, name
            ))?
            .error_for_status()?;
        Ok(resp.json::<T>()?)
    }

    pub fn watch(&self, name: &str) -> Result<impl ResourceWatcher<T>, reqwest::Error> {
        let pre_resp = self
            .rest_client
            .get(format!(
                "{}/{}/{}",
                self.base_path, self.resource_plural, name
            ))?
            .error_for_status()?;

        let pre_obj: T = pre_resp.json()?;
        let resource_version = pre_obj
            .metadata()
            .resource_version
            .clone()
            .unwrap_or_default(); // TODO: missing resourceVersion probably deserves its own error, however we cannot just create a new reqwest::Error it seems

        let resp = self
            .rest_client
            .start_watch(
                format!("{}/{}", self.base_path, self.resource_plural),
                name.to_string(),
                resource_version,
            )?
            .error_for_status()?;

        let reader = std::io::BufReader::new(resp);
        Ok(apimachinery::watch::StreamWatcher::new(reader))
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
