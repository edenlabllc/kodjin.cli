use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

const RESOURCE_COMPARISON_IGNORED_FIELDS: &[&str] = &["/meta/lastUpdated", "/meta/versionId"];

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

pub fn is_resource_changed(mut existing: Value, mut new: Value) -> bool {
    for pointer in RESOURCE_COMPARISON_IGNORED_FIELDS {
        for resource in [&mut existing, &mut new] {
            remove_by_pointer(resource, pointer);

            if let Some((parent_pointer, _)) = pointer.rsplit_once('/') {
                match resource.pointer(parent_pointer) {
                    Some(Value::Object(obj)) if obj.is_empty() => {
                        remove_by_pointer(resource, parent_pointer);
                    }
                    Some(Value::Array(arr)) if arr.is_empty() => {
                        remove_by_pointer(resource, parent_pointer);
                    }
                    _ => (),
                }
            }
        }
    }

    existing != new
}

/// Taken from https://github.com/serde-rs/json/pull/912
pub fn remove_by_pointer(
    value: &mut serde_json::Value,
    pointer: &str,
) -> Option<serde_json::Value> {
    if pointer.is_empty() {
        return Some(value.take());
    }
    #[allow(clippy::manual_split_once)]
    let mut pointer_split = pointer.rsplitn(2, '/');
    let key = pointer_split.next()?;
    let pointer = pointer_split.next()?;
    value.pointer_mut(pointer).and_then(|value| match value {
        serde_json::Value::Object(map) => map.remove(key),
        serde_json::Value::Array(list) => {
            if key.starts_with('+') || (key.starts_with('0') && key.len() != 1) {
                return None;
            }
            key.parse().ok()
        }
        .and_then(move |x| {
            if x < list.len() {
                Some(list.remove(x))
            } else {
                None
            }
        }),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use crate::installer::resource::is_resource_changed;
    use serde_json::json;

    #[test]
    fn patient_only_meta_changed() {
        let a = json!({
          "resourceType": "Patient",
          "extension": [
            {
              "url": "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race",
              "extension": [
                {
                  "url": "ombCategory",
                  "valueCoding": {
                    "system": "urn:oid:2.16.840.1.113883.6.238",
                    "code": "2106-3",
                    "display": "White"
                  }
                },
                {
                  "url": "text",
                  "valueString": "Mixed"
                }
              ]
            },
            {
              "url": "http://hl7.org/fhir/us/core/StructureDefinition/us-core-ethnicity",
              "extension": [
                {
                  "url": "ombCategory",
                  "valueCoding": {
                    "system": "urn:oid:2.16.840.1.113883.6.238",
                    "code": "2186-5",
                    "display": "Not Hispanic or Latino"
                  }
                },
                {
                  "url": "text",
                  "valueString": "Not Hispanic or Latino"
                }
              ]
            }
          ],
          "active": true,
          "name": [
            {
              "family": "Shaw",
              "given": [
                "Mary",
                "A."
              ]
            }
          ],
          "telecom": [
            {
              "system": "phone",
              "value": "555-555-5555",
              "use": "home"
            },
            {
              "system": "email",
              "value": "mary.shaw@example.com",
              "use": "home"
            }
          ],
          "gender": "female",
          "id": "309e53cc-c19f-462b-9c5c-84959ec514d1",
          "meta": {
            "versionId": "1",
            "lastUpdated": "2025-07-16T07:36:16.707981896+00:00"
          }
        });

        let b = json!({
          "resourceType": "Patient",
          "extension": [
            {
              "url": "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race",
              "extension": [
                {
                  "url": "ombCategory",
                  "valueCoding": {
                    "system": "urn:oid:2.16.840.1.113883.6.238",
                    "code": "2106-3",
                    "display": "White"
                  }
                },
                {
                  "url": "text",
                  "valueString": "Mixed"
                }
              ]
            },
            {
              "url": "http://hl7.org/fhir/us/core/StructureDefinition/us-core-ethnicity",
              "extension": [
                {
                  "url": "ombCategory",
                  "valueCoding": {
                    "system": "urn:oid:2.16.840.1.113883.6.238",
                    "code": "2186-5",
                    "display": "Not Hispanic or Latino"
                  }
                },
                {
                  "url": "text",
                  "valueString": "Not Hispanic or Latino"
                }
              ]
            }
          ],
          "active": true,
          "name": [
            {
              "family": "Shaw",
              "given": [
                "Mary",
                "A."
              ]
            }
          ],
          "telecom": [
            {
              "system": "phone",
              "value": "555-555-5555",
              "use": "home"
            },
            {
              "system": "email",
              "value": "mary.shaw@example.com",
              "use": "home"
            }
          ],
          "gender": "female",
          "id": "309e53cc-c19f-462b-9c5c-84959ec514d1"
        });

        assert!(!is_resource_changed(a, b));
    }
}
