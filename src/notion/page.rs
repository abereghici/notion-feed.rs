use super::{
    database::DatabaseKind,
    models::{Page, Parent, PropertyValue},
    Client,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

#[derive(Debug, Serialize, Deserialize)]
struct CreatePageProps {
    parent: Parent,
    properties: HashMap<String, PropertyValue>,
}

impl<'a> Client<'a> {
    pub async fn create_page(
        &self,
        kind: DatabaseKind,
        properties: HashMap<String, PropertyValue>,
    ) -> Result<Page, Box<dyn Error>> {
        let db_id = match kind {
            DatabaseKind::Source => &self.config.notion_source_database_id,
            DatabaseKind::Feed => &self.config.notion_feed_database_id,
        };

        let path = "/pages";

        let create_page_props = CreatePageProps {
            parent: Parent {
                parent_type: "database_id".to_string(),
                database_id: Some(db_id.to_string()),
            },
            properties,
        };

        let res = self
            .build_request(Method::POST, path)
            .json(&create_page_props)
            .send()
            .await?
            .error_for_status()?;

        Ok(res.json::<Page>().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        notion::models::{RichText, Text},
    };

    #[tokio::test]
    async fn create_page() {
        let config = Config::new(None, None).unwrap();
        let client = Client::new(&config).unwrap();

        let page_props = HashMap::from([
            (
                "Title".to_string(),
                PropertyValue::Title {
                    title: vec![RichText::Text {
                        rich_text: None,
                        text: Text {
                            content: String::from("Test Link"),
                            link: None,
                        },
                    }],
                },
            ),
            (
                "Link".to_string(),
                PropertyValue::Url {
                    url: Some(String::from("https://bereghici.dev")),
                },
            ),
            (
                "Enabled".to_string(),
                PropertyValue::Checkbox { checkbox: false },
            ),
        ]);

        let page = client.create_page(DatabaseKind::Source, page_props).await;

        assert!(page.is_ok());
    }
}
