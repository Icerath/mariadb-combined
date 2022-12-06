use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KbItem {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "Include")]
    pub pdf_include: u8,
}

pub fn read_kb_urls() -> Vec<KbItem> {
    csv::Reader::from_path("../../kb_urls.csv")
        .expect("Failed to Read: 'kb_urls.csv'")
        .deserialize::<KbItem>()
        .filter_map(Result::ok)
        .collect()
}
