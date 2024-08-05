mod capability_statement;

use anyhow::{anyhow, Context};
use capability_statement::CapabilityStatement;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct FhirClient {
    client: ureq::Agent,
    base_url: String,
}

impl FhirClient {
    pub fn new(url: String) -> Self {
        Self {
            client: ureq::Agent::new(),
            base_url: url,
        }
    }

    /// Standard FHIR JSON request
    fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> anyhow::Result<T> {
        let url = format!("{}{path}", self.base_url);

        let request = self.client.request(method, &url);

        let response = match body {
            Some(body) => request
                .set("Content-Type", "application/json")
                .send_string(body),
            _ => request.call(),
        }
        .context("Request error")?;

        if response.status() % 200 != 0 {
            // TODO extract error from body
            return Err(anyhow!(
                "Got error code from server: {} {}",
                response.status(),
                response.status_text()
            ));
        }

        let raw = response.into_string().context("Could not read response")?;
        serde_json::from_str(&raw).context("Could not parse response")
    }

    pub fn upsert(&self, resource_type: &str, id: &str, payload: &str) -> anyhow::Result<Value> {
        self.request("PUT", &format!("/{resource_type}/{id}"), Some(payload))
    }

    pub fn get_metadata(&self) -> anyhow::Result<CapabilityStatement> {
        self.request("GET", "/metadata", None)
    }
}
