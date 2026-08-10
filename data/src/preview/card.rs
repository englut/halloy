use serde::{Deserialize, Serialize};
use url::Url;

use super::Image;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card<'a> {
    pub url: Url,
    pub canonical_url: Url,
    pub image: Image<'a>,
    pub title: String,
    pub description: Option<String>,
}
