use crate::notion::models::{Page, PropertyValue};

#[derive(Debug)]
pub struct FeedItem {
    link: String,
}

impl FeedItem {
    pub fn new(page: &Page) -> Option<FeedItem> {
        let properties = page.properties.as_ref()?;

        let link = match properties.get("Link") {
            Some(PropertyValue::Url { url }) => match url {
                Some(url) => Some(url.to_string()),
                None => None,
            },
            _ => None,
        };

        if let Some(link) = link {
            return Some(Self { link });
        }

        None
    }
}
