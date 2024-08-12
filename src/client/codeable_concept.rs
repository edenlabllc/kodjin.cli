use super::coding::Coding;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CodeableConcept {
    #[serde(default)]
    pub coding: Vec<Coding>,
    pub text: Option<String>,
}
