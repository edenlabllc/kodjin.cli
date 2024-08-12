use super::codeable_concept::CodeableConcept;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OperationOutcome {
    #[serde(rename = "issue")]
    pub issues: Vec<Issue>,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub code: IssueCode,
    pub details: Option<CodeableConcept>,
    pub diagnostics: Option<String>,
    #[serde(default)]
    pub location: Vec<String>,
    #[serde(default)]
    pub expression: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum IssueSeverity {
    Error,
    Fatal,
    Warning,
    Information,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize)]
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
