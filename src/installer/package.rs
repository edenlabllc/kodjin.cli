use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize};

pub struct FhirPackage {
    pub dir: PathBuf,
}

impl FhirPackage {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> anyhow::Result<T> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Could not read {}", path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Could not parse file {}", path.display()))
    }

    pub fn read_manifest(&self) -> anyhow::Result<PackageManifest> {
        self.read_json(&self.dir.join("package").join("package.json"))
    }

    pub fn read_index(&self) -> anyhow::Result<PackageIndex> {
        let index_path = self.dir.join("package").join(".index.json");
        if index_path.exists() {
            self.read_json(&index_path)
        } else {
            let mut files = Vec::new();

            let entries = fs::read_dir(self.dir.join("package"))?;
            for result in entries {
                let entry = result?;
                if let Ok(resource_info) = self.read_json::<ResourceTypeId>(&entry.path()) {
                    let filename = entry.file_name().to_string_lossy().into_owned();

                    files.push(PackageIndexFile {
                        filename,
                        filepath: None,
                        resource_type: resource_info.resource_type,
                        id: resource_info.id,
                    });
                }
            }

            Ok(PackageIndex { files })
        }
    }
}

#[derive(Deserialize)]
pub struct PackageManifest {
    pub name: String,
    // pub version: String,
    // pub url: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct PackageIndex {
    pub files: Vec<PackageIndexFile>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageIndexFile {
    filename: String,
    filepath: Option<String>,
    pub resource_type: String,
    pub id: String,
}

impl PackageIndexFile {
    pub fn get_path(&self) -> PathBuf {
        if let Some(path) = &self.filepath {
            PathBuf::from(path)
        } else {
            PathBuf::from("package").join(&self.filename)
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ResourceTypeId {
    pub resource_type: String,
    pub id: String,
}
