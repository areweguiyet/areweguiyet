#[derive(Debug, Deserialize, Serialize)]
pub struct NewsfeedEntry {
    pub title: String,
    pub author: String,
    pub order: u32,
    pub source: NewsfeedSource,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum NewsfeedSource {
    Link {
        link: String,
    },
    Post {
        /// File name with no associated path
        file_name: String,
    }
}