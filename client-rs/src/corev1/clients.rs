use crate::{rest::RestClient};
use k8s_openapi::serde;

use k8s_openapi::api::core::v1::Pod;

pub struct CoreV1Client<'c> {
    rest_client: &'c crate::rest::RestClient,
}

impl<'c> CoreV1Client<'c> {
    pub fn new(rest_client: &'c crate::rest::RestClient) -> Self {
        CoreV1Client {
            rest_client,
        }
    }

    pub fn rest_client(&self) -> &crate::rest::RestClient {
        self.rest_client
    }

    pub fn pods(&self, namespace: &str) -> PodsClient<'_> {
        new_pods_client(self.rest_client, namespace)
    }
}

pub struct ResourceClient<'c, T> {
    _client_type: std::marker::PhantomData<T>,

    rest_client: &'c RestClient,
    resource_plural: String,
    base_path: String,
}

impl<'c, T> ResourceClient<'c, T> {
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
    pub fn get(&self, name: &str) -> Result<T, reqwest::Error> where T: serde::de::DeserializeOwned {
        let resp = self.rest_client.get(
            format!("{}/{}/{}", self.base_path, self.resource_plural, name)
        )?.error_for_status()?;
        Ok(resp.json::<T>()?)
    }
}

pub type PodsClient<'c> = ResourceClient<'c, Pod>;

fn new_pods_client<'c>(client: &'c RestClient, namespace: &str) -> ResourceClient<'c, Pod> {
    ResourceClient::new(client, "pods".to_string(), Some(namespace.to_string()))
}