use super::{models::Page, Client};
use reqwest::{Error, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub enum DatabaseKind {
    Source,
    Feed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSort {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    /// created_time or last_edited_time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// ascending or descending
    pub direction: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    #[serde(rename = "rich_text")]
    Text {
        equals: String,
    },
    Checkbox {
        equals: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub property: String,
    #[serde(flatten)]
    pub kind: FilterKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatabaseFilter {
    Property {
        #[serde(flatten)]
        filter: Filter,
    },
    Compound {
        #[serde(flatten)]
        filter: HashMap<String, Vec<Filter>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<DatabaseFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorts: Option<Vec<DatabaseSort>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pages {
    pub object: String,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub results: Vec<Page>,
}

impl<'a> Client<'a> {
    pub async fn query_database(
        &self,
        kind: DatabaseKind,
        query: Option<DatabaseQuery>,
    ) -> Result<Pages, Error> {
        let db_id = match kind {
            DatabaseKind::Source => &self.config.notion_source_database_id,
            DatabaseKind::Feed => &self.config.notion_feed_database_id,
        };

        let path = format!("/databases/{}/query", db_id);
        let mut req = self.build_request(Method::POST, path);

        if let Some(query) = query {
            req = req.json(&query);
        }

        let res = req.send().await?.error_for_status()?;

        Ok(res.json::<Pages>().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn it_query_database() {
        let config = Config::new(None, None).unwrap();
        let client = Client::new(&config).unwrap();

        let filter = DatabaseFilter::Property {
            filter: Filter {
                property: String::from("Title"),
                kind: FilterKind::Text {
                    equals: String::from("Javascript Weekly"),
                },
            },
        };

        let query = DatabaseQuery {
            start_cursor: None,
            page_size: None,
            filter: Some(filter),
            sorts: None,
        };

        let pages = client
            .query_database(DatabaseKind::Source, Some(query))
            .await;

        assert!(pages.is_ok());
    }
}
