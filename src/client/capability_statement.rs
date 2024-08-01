use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "resourceType", rename_all = "camelCase")]
pub struct CapabilityStatement {
    pub id: String,
    pub url: Option<String>,
    pub version: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub experimental: Option<bool>,
    pub date: String, //bson::DateTime,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub purpose: Option<String>,
    pub copyright: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub instantiates: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub imports: Vec<String>,
    pub software: Option<Software>,
    pub fhir_version: String,
    pub format: Option<Vec<String>>,
    pub patch_format: Option<Vec<String>>,
    pub implementation_guide: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub version: Option<String>,
    pub name: String,
    pub release_date: Option<String>,
}
