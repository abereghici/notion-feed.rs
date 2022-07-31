use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct RichTextProperties {
    pub plain_text: String,
    pub href: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Link {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Text {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Link>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum RichText {
    Text {
        #[serde(flatten)]
        rich_text: Option<RichTextProperties>,
        text: Text,
    },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum PropertyValue {
    Title {
        title: Vec<RichText>,
    },
    #[serde(rename = "rich_text")]
    Text {
        rich_text: Vec<RichText>,
    },
    Url {
        url: Option<String>,
    },
    Checkbox {
        checkbox: bool,
    },
    Formula {
        expression: Option<String>,
    },
    CreatedTime {
        created_time: DateTime<Utc>,
    },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Parent {
    #[serde(rename = "type")]
    pub parent_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Page {
    pub id: String,
    pub archived: bool,
    pub parent: Option<Parent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertyValue>>,
}
