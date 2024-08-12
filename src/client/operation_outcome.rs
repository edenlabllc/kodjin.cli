use super::codeable_concept::CodeableConcept;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Debug, Serialize)]
pub struct OperationOutcome {
    #[serde(rename = "issue", skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<Issue>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Debug)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub code: IssueCode,
    pub details: Option<CodeableConcept>,
    pub diagnostics: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub location: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expression: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum IssueSeverity {
    Error,
    Fatal,
    Warning,
    Information,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueCode {
    Exception,
    Structure,
    Required,
    Value,
    Invalid,
    NotFound,
    NotSupported,
    NoStore,
    Login,
    Forbidden,
    Informational,
    Processing,
    Duplicate,
    CodeInvalid,
    Invariant,
    Conflict,
    TooLong,
    Transient,
    TooCostly,
}
