mod bundle;
mod capability_statement;
mod codeable_concept;
mod coding;
pub mod operation_outcome;

use anyhow::{anyhow, Context};
use bundle::Bundle;
use capability_statement::CapabilityStatement;
use colored_json::ToColoredJson;
use operation_outcome::OperationOutcome;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, RequestBuilder, Response, StatusCode, Url,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct FhirClient {
    client: reqwest::Client,
    base_url: Arc<str>,
    search_url: Arc<str>,
}

impl FhirClient {
    pub fn new(
        url: String,
        search_url: Option<String>,
        insecure_certificates: bool,
        timeout: Duration,
        headers: &[String],
    ) -> anyhow::Result<Self> {
        let mut header_map = HeaderMap::new();

        for header in headers {
            let (key, value) = header
                .split_once(':')
                .context("Header param must contain ':'")?;

            header_map.insert(
                HeaderName::from_str(key)?,
                HeaderValue::from_str(value.trim_ascii_start())?,
            );
        }

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(insecure_certificates)
            .default_headers(header_map)
            .timeout(timeout)
            .build()
            .unwrap();

        Ok(Self {
            client,
            base_url: url.as_str().into(),
            search_url: search_url.unwrap_or(url).into(),
        })
    }

    /// Standard FHIR JSON request
    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        self.client.request(method, url)
    }

    fn search_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.search_url);
        self.client.request(method, url)
    }

    pub async fn upsert(
        &self,
        resource_type: &str,
        id: &str,
        payload: &impl Serialize,
    ) -> Result<Value, FhirError> {
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

    pub async fn snapshot(&self, payload: &impl Serialize) -> Result<Value, FhirError> {
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
    ) -> Result<Bundle<T>, FhirError> {
        let response = self
            .search_request(Method::GET, &format!("/{resource_type}"))
            .query(params)
            .header("Prefer", "handling=strict")
            .send()
            .await?;
        let bundle = handle_response_error(response).await?.json().await?;
        Ok(bundle)
    }

    pub async fn delete(&self, resource_type: &str, id: &str) -> Result<(), FhirError> {
        self.request(Method::DELETE, &format!("/{resource_type}/{id}"))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_metadata(&self) -> Result<CapabilityStatement, FhirError> {
        Ok(self
            .request(Method::GET, "/metadata")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

async fn handle_response_error(response: Response) -> Result<Response, FhirError> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status();
        let url = response.url().clone();
        let body = response.text().await?;
        match serde_json::from_str::<OperationOutcome>(&body) {
            Ok(outcome) => Err(FhirError::Outcome {
                status,
                outcome,
                url,
            }),
            Err(_) => Err(FhirError::Other(anyhow!(
                "Server error (status {status}): \"{body}\""
            ))),
        }
    }
}

#[derive(Debug)]
pub enum FhirError {
    Outcome {
        status: StatusCode,
        outcome: OperationOutcome,
        url: Url,
    },
    Other(anyhow::Error),
}

impl fmt::Display for FhirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FhirError::Outcome {
                status,
                outcome,
                url,
            } => {
                write!(
                    f,
                    "FHIR error (status {status} at {url}):\n{}",
                    serde_json::to_string_pretty(&outcome)
                        .unwrap()
                        .to_colored_json_auto()
                        .unwrap()
                )
            }
            FhirError::Other(error) => write!(f, "{error:#}"),
        }
    }
}

impl From<reqwest::Error> for FhirError {
    fn from(err: reqwest::Error) -> Self {
        Self::Other(err.into())
    }
}

impl std::error::Error for FhirError {}
