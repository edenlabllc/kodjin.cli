use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

pub struct Resource {
    pub data: Value,
    pub info: ResourceInfo,
    /// Relative path where the resource was loaded from
    pub source_path: PathBuf,
}

impl Resource {
    pub fn set_id(&mut self, new_id: String) {
        self.info.id.clone_from(&new_id);
        if let Some(obj) = self.data.as_object_mut() {
            obj.insert("id".to_owned(), new_id.into());
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    pub resource_type: String,
    pub id: String,
    pub url: Option<String>,
    pub version: Option<String>,
}

impl ResourceInfo {
    pub fn canonical_url(&self) -> Option<String> {
        match &self.url {
            Some(url) => match &self.version {
                Some(version) => Some(format!("{url}|{version}")),
                None => Some(url.clone()),
            },
            None => None,
        }
    }
}
