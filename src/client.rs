mod bundle;
mod capability_statement;
mod codeable_concept;
mod coding;
mod operation_outcome;

use anyhow::anyhow;
use bundle::Bundle;
use capability_statement::CapabilityStatement;
use colored_json::ToColoredJson;
use operation_outcome::OperationOutcome;
use reqwest::{Method, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct FhirClient {
    client: reqwest::Client,
    base_url: Arc<str>,
}

impl FhirClient {
    pub fn new(url: String, insecure_certificates: bool) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(insecure_certificates)
            .build()
            .unwrap();
        Self {
            client,
            base_url: url.into(),
        }
    }

    /// Standard FHIR JSON request
    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        self.client.request(method, url)
    }

    pub async fn upsert(
        &self,
        resource_type: &str,
        id: &str,
        payload: &impl Serialize,
    ) -> anyhow::Result<Value> {
        let response = self
            .request(Method::PUT, &format!("/{resource_type}/{id}"))
            .json(payload)
            .send()
            .await?;
        Ok(handle_response_error(response).await?.json().await?)
    }

    /*pub async fn get(&self, resource_type: &str, id: &str) -> anyhow::Result<Option<Value>> {
        let response = self
            .request(Method::GET, &format!("/{resource_type}/{id}"))
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Ok(Some(handle_response_error(response).await?.json().await?))
        }
    }*/

    pub async fn snapshot(&self, payload: &impl Serialize) -> anyhow::Result<Value> {
        let response = self
            .request(Method::POST, "/StructureDefinition/$snapshot")
            .json(payload)
            .send()
            .await?;
        Ok(handle_response_error(response).await?.json().await?)
    }

    pub async fn search<T: DeserializeOwned>(
        &self,
        resource_type: &str,
        params: &[(&str, &str)],
    ) -> anyhow::Result<Bundle<T>> {
        let response = self
            .request(Method::GET, &format!("/{resource_type}"))
            .query(params)
            .header("Prefer", "handling=strict")
            .send()
            .await?;
        let bundle = handle_response_error(response).await?.json().await?;
        Ok(bundle)
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

async fn handle_response_error(response: Response) -> anyhow::Result<Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status();
        let url = response.url().clone();
        let body = response.text().await?;
        match serde_json::from_str::<OperationOutcome>(&body) {
            Ok(outcome) => Err(anyhow!(
                "FHIR error (status {status} at {url}):\n{}",
                serde_json::to_string_pretty(&outcome)
                    .unwrap()
                    .to_colored_json_auto()?
            )),
            Err(_) => Err(anyhow!("Server error (status {status}): \"{body}\"")),
        }
    }
}
