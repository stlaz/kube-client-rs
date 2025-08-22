use reqwest::blocking::{ClientBuilder, Client, Request, Response};
use reqwest::{ Result, Url};

use std::sync::Arc;

use crate::rest::config::Config;

pub trait RoundTripper {
    fn round_trip(&self, req: Request) -> Result<Response>;
}
type DynamicRoundTripper = Arc<dyn RoundTripper>;

pub struct HTTPClient {
    pub client: Client,
}

impl RoundTripper for HTTPClient {
    fn round_trip(&self, req: Request) -> Result<Response> {
        self.client.execute(req)
    }
}

pub struct RestClient {
    transport: DynamicRoundTripper,
    base_url: String,
}

impl RestClient {
    pub fn new(client: DynamicRoundTripper, base_url: &str) -> Self {
        RestClient {
            transport: client,
            base_url: base_url.to_string(),
        }
    }

    pub fn get(&self, path: String) -> Result<Response> {
        let url = Url::parse(&format!("{}{}", self.base_url, path))
            .expect("Failed to parse URL");
        let req = Request::new(reqwest::Method::GET, url);
        self.transport.round_trip(req)
    }
}

// TODO: hide and add RestClientFor method?
struct BearerTokenClient {
    client: DynamicRoundTripper,
    token: String,
}

impl BearerTokenClient {
    fn new(client: DynamicRoundTripper, token: &str) -> Self {
        BearerTokenClient {
            client,
            token: token.to_string(),
        }
    }
}

impl RoundTripper for BearerTokenClient {
    fn round_trip(&self, req: Request) -> Result<Response> {
        let mut req = req;
        req.headers_mut().insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        self.client.round_trip(req)
    }
}

pub fn rest_client_for(config: &Config) -> RestClient {
    let rt: DynamicRoundTripper = Arc::new(
        HTTPClient { client: ClientBuilder::new()
            .user_agent(config.user_agent.clone().unwrap_or_else(|| "kube-client-rs".to_string()))
            .danger_accept_invalid_certs(true) // FIXME
            .build()
            .expect("Failed to build HTTP client") }
    );

    let rt = if let Some(token) = &config.bearer_token {
        Arc::new(BearerTokenClient::new(rt, token))
    } else {
        rt
    };
    RestClient::new(rt, config.base_url.as_str())
}