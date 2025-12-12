use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bundle<T> {
    // pub r#type: BundleType,
    #[serde(
        default = "Vec::<BundleEntry<T>>::new",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub entry: Vec<BundleEntry<T>>,
    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    // pub link: Vec<Link>,
    // pub id: Option<String>,
    // pub language: Option<String>,
    // pub timestamp: Option<String>,
}

/*#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum BundleType {
    Document,
    Message,
    Transaction,
    TransactionResponse,
    Batch,
    BatchResponse,
    History,
    Searchset,
    Collection,
}*/

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry<T> {
    pub full_url: Option<String>,
    pub resource: Option<T>,
    pub search: Option<EntrySearch>,
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct Link {
//     pub relation: String,
//     pub url: String,
// }

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct EntrySearch {
    pub mode: Option<SearchMode>,
    pub score: Option<f64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchMode {
    Match,
    Include,
    Outcome,
}
