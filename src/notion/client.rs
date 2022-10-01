use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    IntoUrl, Method, RequestBuilder,
};
use std::{error::Error, str::FromStr};

use crate::config::Config;

const BASE_API: &str = "https://api.notion.com/v1";
const VERSION: &str = "2022-02-22";

#[derive(Debug)]
pub struct Client<'a> {
    pub config: &'a Config,
    client: reqwest::Client,
}

impl<'a> Client<'a> {
    pub fn new(config: &'a Config) -> Result<Client, Box<dyn Error>> {
        let headers = HeaderMap::from_iter(vec![
            (
                HeaderName::from_str("Authorization")?,
                HeaderValue::from_str(format!("Bearer {}", config.notion_api_token).as_str())?,
            ),
            (
                HeaderName::from_str("Notion-Version")?,
                HeaderValue::from_str(&VERSION)?,
            ),
        ]);

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Self { config, client })
    }

    pub fn build_request<U: IntoUrl>(&self, method: Method, path: U) -> RequestBuilder {
        let url = format!("{}{}", &BASE_API, path.as_str());
        self.client.request(method, url)
    }
}
