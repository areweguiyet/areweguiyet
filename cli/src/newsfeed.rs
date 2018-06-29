#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum NewsfeedEntry {
    Link {
        title: String,
        author: String,
        link: String,
    }
}
