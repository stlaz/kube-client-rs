use k8s_openapi::serde;

use std::ops::Deref;

use k8s_openapi::List;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use apimachinery::watch::ResourceWatcher;

use crate::rest::RestClient;

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
