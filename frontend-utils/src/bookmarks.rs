mod read;
pub use read::read_bookmarks;

use url::Url;

pub static INVALID_URL: &str = "invalid:///";

#[derive(Debug, PartialEq)]
pub struct Bookmark {
    pub url: Url,
    pub name: String,
}

impl Bookmark {
    pub fn is_invalid(&self) -> bool {
        self.url.as_str() == INVALID_URL
    }
}

pub type Bookmarks = Vec<Bookmark>;
