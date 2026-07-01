mod bundle;
mod capability_statement;
mod codeable_concept;
mod coding;
pub mod operation_outcome;
mod parameters;

pub use parameters::Parameters;

use anyhow::{anyhow, Context};
use bundle::Bundle;
use capability_statement::CapabilityStatement;
use colored_json::ToColoredJson;
use oauth2::{ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use operation_outcome::OperationOutcome;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, RequestBuilder, Response, StatusCode, Url,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr, sync::Arc, time::Duration};

use crate::args::{self, Args, AuthOptions};

pub struct FhirClient {
    client: reqwest::Client,
    base_url: Arc<str>,
    search_url: Arc<str>,
    auth: Option<ConfiguredAuth>,
    multitenancy_header_names: Vec<String>,
}

impl FhirClient {
    const ASTERISK_MULTITENANCY_VALUE: &'static str = "*";

    pub async fn new(
        url: String,
        search_url: Option<String>,
        args: &Args,
        timeout: Duration,
    ) -> anyhow::Result<Self> {
        let mut header_map = HeaderMap::new();

        for header in &args.header {
            let (key, value) = header
                .split_once(':')
                .context("Header param must contain ':'")?;

            header_map.insert(
                HeaderName::from_str(key)?,
                HeaderValue::from_str(value.trim_ascii_start())?,
            );
        }

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(args.insecure_certificates)
            .default_headers(header_map)
            .timeout(timeout)
            .build()
            .unwrap();

        Ok(Self {
            client,
            auth: configure_auth(args).await?,
            base_url: url.as_str().into(),
            search_url: search_url.unwrap_or(url).into(),
            multitenancy_header_names: args.multitenancy_header_names.clone(),
        })
    }

    /// Standard FHIR JSON request
    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        let mut builder = self.client.request(method, url);
        builder = add_auth(builder, &self.auth);

        builder
    }

    fn search_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.search_url);
        let mut builder = self.client.request(method, url);
        builder = add_auth(builder, &self.auth);

        builder
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

    pub async fn start_reindex(&self) -> Result<Parameters, FhirError> {
        let body = serde_json::json!({ "resourceType": "Parameters" });

        let mut headers = HeaderMap::new();
        for name in &self.multitenancy_header_names {
            headers.insert(
                HeaderName::from_str(name).map_err(|e| FhirError::Other(e.into()))?,
                Self::ASTERISK_MULTITENANCY_VALUE.parse().unwrap(),
            );
        }

        let response = self
            .request(Method::POST, "/$reindex")
            .json(&body)
            .headers(headers)
            .send()
            .await?;
        Ok(handle_response_error(response).await?.json().await?)
    }

    pub async fn get_reindex_status(&self, id: &str) -> Result<Parameters, FhirError> {
        let mut headers = HeaderMap::new();
        for name in &self.multitenancy_header_names {
            headers.insert(
                HeaderName::from_str(name).map_err(|e| FhirError::Other(e.into()))?,
                Self::ASTERISK_MULTITENANCY_VALUE.parse().unwrap(),
            );
        }

        let response = self
            .request(Method::GET, &format!("/reindex/{id}"))
            .headers(headers)
            .send()
            .await?;
        Ok(handle_response_error(response).await?.json().await?)
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

enum ConfiguredAuth {
    Basic {
        user: String,
        password: Option<String>,
    },
    Bearer(String),
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

async fn configure_auth(args: &Args) -> anyhow::Result<Option<ConfiguredAuth>> {
    let AuthOptions {
        user,
        password,
        bearer,
        token_url,
        client_id,
        client_secret,
        scope,
    } = args.auth_options.clone();

    let auth = match args.auth {
        Some(args::Auth::Basic) => {
            let user = user.context("user must be specified when using basic auth")?;

            Some(ConfiguredAuth::Basic { user, password })
        }
        Some(args::Auth::Bearer) => {
            let token = bearer.context("Bearer param must be specified when using bearer auth")?;
            Some(ConfiguredAuth::Bearer(token))
        }
        Some(args::Auth::Oauth) => {
            let client_id = client_id.context("client id must be specified for oauth")?;
            let client_secret =
                client_secret.context("client secret must be specified for oauth")?;
            let token_url = token_url.context("token URL must be specified for oauth")?;

            let oauth_client = oauth2::basic::BasicClient::new(ClientId::new(client_id))
                .set_client_secret(ClientSecret::new(client_secret))
                .set_token_uri(TokenUrl::new(token_url)?);

            let http_client = reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;

            let token_result = oauth_client
                .exchange_client_credentials()
                .add_scopes(scope.into_iter().map(Scope::new))
                .request_async(&http_client)
                .await?;

            let token = token_result.access_token().secret().clone();

            Some(ConfiguredAuth::Bearer(token))
        }
        None => None,
    };
    Ok(auth)
}

fn add_auth(mut builder: RequestBuilder, auth: &Option<ConfiguredAuth>) -> RequestBuilder {
    if let Some(auth) = auth {
        match auth {
            ConfiguredAuth::Basic { user, password } => {
                builder = builder.basic_auth(user, password.as_ref());
            }
            ConfiguredAuth::Bearer(token) => {
                builder = builder.bearer_auth(token);
            }
        }
    }

    builder
}
