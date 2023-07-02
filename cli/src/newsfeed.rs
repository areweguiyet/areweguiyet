#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NewsfeedCommon {
    pub title: String,
    pub author: String,
    pub date: toml::value::Datetime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewsfeedLink {
    pub link: String,
    #[serde(flatten)]
    pub common: NewsfeedCommon,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewsfeedPost {
    /// File name with no associated path
    #[serde(rename = "file-name")]
    pub file_name: String,
    #[serde(flatten)]
    pub common: NewsfeedCommon,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Newsfeed {
    pub links: Vec<NewsfeedLink>,
    pub posts: Vec<NewsfeedPost>,
}
