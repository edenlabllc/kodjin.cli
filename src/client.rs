mod capability_statement;

use anyhow::Context;
use capability_statement::CapabilityStatement;
use reqwest::{Method, Response};
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
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&str>,
    ) -> anyhow::Result<Response> {
        let url = format!("{}{path}", self.base_url);

        let mut request = self.client.request(method, &url);

        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .body(body.to_owned());
        }

        let response = request
            .send()
            .await
            .context("Request error")?
            .error_for_status()?;

        Ok(response)
    }

    pub async fn upsert(
        &self,
        resource_type: &str,
        id: &str,
        payload: &str,
    ) -> anyhow::Result<Value> {
        Ok(self
            .request(
                Method::PUT,
                &format!("/{resource_type}/{id}"),
                Some(payload),
            )
            .await?
            .json()
            .await?)
    }

    pub async fn delete(&self, resource_type: &str, id: &str) -> anyhow::Result<()> {
        self.request(Method::DELETE, &format!("/{resource_type}/{id}"), None)
            .await?;
        Ok(())
    }

    pub async fn get_metadata(&self) -> anyhow::Result<CapabilityStatement> {
        Ok(self
            .request(Method::GET, "/metadata", None)
            .await?
            .json()
            .await?)
    }
}
