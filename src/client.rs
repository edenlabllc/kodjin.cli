mod capability_statement;

use anyhow::{anyhow, Context};
use capability_statement::CapabilityStatement;
use serde::de::DeserializeOwned;

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
    fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{path}", self.base_url);
        let response = self.client.get(&url).call().context("Request error")?;
        if response.status() % 200 != 0 {
            // TODO extract error from body
            return Err(anyhow!(
                "Got error code from server: {} {}",
                response.status(),
                response.status_text()
            ));
        }

        response
            .into_json()
            .context("Failed to parse response as JSON")
    }

    pub fn get_metadata(&self) -> anyhow::Result<CapabilityStatement> {
        self.get("/metadata")
    }
}
