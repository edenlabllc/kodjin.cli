use anyhow::Context;
use deno_npm::registry::NpmPackageInfo;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct RegistryClient {
    pub client: reqwest::Client,
    pub base_url: Arc<str>,
}

impl RegistryClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: url.into(),
        }
    }

    pub async fn package_info(&self, name: &str) -> anyhow::Result<NpmPackageInfo> {
        let url = format!("{}/{name}", self.base_url);
        let response = self.client.get(url).send().await?.error_for_status()?;

        let mut data_raw: Value = response.json().await.context("Could not parse response")?;

        // Fix missing fields in response
        if let Some(Value::Object(versions)) = data_raw.get_mut("versions") {
            for version in versions.values_mut() {
                if let Some(Value::Object(dist)) = version.get_mut("dist") {
                    dist.entry("shasum").or_insert_with(|| String::new().into());
                }
            }
        }

        serde_json::from_value(data_raw).context("Could not parse package info")
    }
}
