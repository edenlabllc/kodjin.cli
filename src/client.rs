mod capability_statement;

use capability_statement::CapabilityStatement;
use reqwest::{Method, RequestBuilder, StatusCode};
use serde_json::Value;

pub struct FhirClient {
    client: reqwest::Client,
    base_url: String,
}

impl FhirClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: url,
        }
    }

    /// Standard FHIR JSON request
    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        self.client.request(method, &url)
    }

    pub async fn upsert(
        &self,
        resource_type: &str,
        id: &str,
        payload: &str,
    ) -> anyhow::Result<Value> {
        Ok(self
            .request(Method::PUT, &format!("/{resource_type}/{id}"))
            .body(payload.to_owned())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn get(&self, resource_type: &str, id: &str) -> anyhow::Result<Option<Value>> {
        let response = self
            .request(Method::GET, &format!("/{resource_type}/{id}"))
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Ok(Some(response.error_for_status()?.json().await?))
        }
    }

    pub async fn delete(&self, resource_type: &str, id: &str) -> anyhow::Result<()> {
        self.request(Method::DELETE, &format!("/{resource_type}/{id}"))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_metadata(&self) -> anyhow::Result<CapabilityStatement> {
        Ok(self
            .request(Method::GET, "/metadata")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}
