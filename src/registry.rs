pub struct RegistryClient {
    client: reqwest::Client,
    base_url: String,
}

impl RegistryClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: url,
        }
    }
}
