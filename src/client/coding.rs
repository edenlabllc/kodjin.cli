use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Coding {
    pub system: Option<String>,
    pub version: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
    pub user_selected: Option<bool>,
}
