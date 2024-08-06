mod capability_statement;

use anyhow::{anyhow, Context};
use capability_statement::CapabilityStatement;
use serde_json::Value;
use ureq::Response;

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
    fn request(&self, method: &str, path: &str, body: Option<&str>) -> anyhow::Result<Response> {
        let url = format!("{}{path}", self.base_url);

        let request = self.client.request(method, &url);

        let response = match body {
            Some(body) => request
                .set("Content-Type", "application/json")
                .send_string(body),
            _ => request.call(),
        }
        .context("Request error")?;

        if !(200..300).contains(&response.status()) {
            // TODO extract error from body
            return Err(anyhow!(
                "Got error code from server: {} {}",
                response.status(),
                response.status_text()
            ));
        }

        Ok(response)
    }

    pub fn upsert(&self, resource_type: &str, id: &str, payload: &str) -> anyhow::Result<Value> {
        Ok(self
            .request("PUT", &format!("/{resource_type}/{id}"), Some(payload))?
            .into_json()?)
    }

    pub fn delete(&self, resource_type: &str, id: &str) -> anyhow::Result<()> {
        self.request("DELETE", &format!("/{resource_type}/{id}"), None)?;
        Ok(())
    }

    pub fn get_metadata(&self) -> anyhow::Result<CapabilityStatement> {
        Ok(self.request("GET", "/metadata", None)?.into_json()?)
    }
}
