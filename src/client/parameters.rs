use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    pub parameter: Vec<Parameter>,
}

#[derive(Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "valueString")]
    pub value_string: String,
}

impl Parameters {
    pub fn get_value(&self, name: &str) -> Option<&str> {
        self.parameter
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value_string.as_str())
    }
}
