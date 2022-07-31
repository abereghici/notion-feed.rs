use crate::notion::{
    database::{DatabaseFilter, DatabaseKind, DatabaseQuery, Filter, FilterKind},
    models::{Page, PropertyValue, RichText, Text},
    Client,
};
use futures::{future, try_join};
use rss::Channel;
use std::{collections::HashMap, convert::identity, error::Error};

pub struct Feed<'a> {
    client: &'a Client<'a>,
}

impl<'a> Feed<'a> {
    pub fn new(client: &'a Client<'a>) -> Feed<'a> {
        Feed { client }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let (source_list, feed_list) = try_join!(self.get_source_list(), self.get_feed_list())?;

        let channels = future::join_all(
            source_list
                .into_iter()
                .map(|feed| Feed::<'_>::get_rss_items(feed)),
        )
        .await;

        let mut all_items = vec![];

        channels.iter().for_each(|channel| {
            if let Ok(channel) = channel {
                all_items.extend(&channel.items);
            }
        });

        future::join_all(
            all_items
                .iter()
                .filter(|item| match &item.link {
                    Some(link) => !feed_list.contains(link),
                    None => false,
                })
                .map(|item| {
                    let title = &item.title;
                    let link = &item.link;

                    if let (Some(title), Some(link)) = (title, link) {
                        return Some(self.add_reader_entry(title.to_string(), link.to_string()));
                    }
                    None
                })
                .filter_map(identity),
        )
        .await;

        Ok(())
    }

    pub async fn get_source_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let filter = DatabaseFilter::Compound {
            filter: HashMap::from([(
                "or".to_string(),
                vec![Filter {
                    property: "Enabled".to_string(),
                    kind: FilterKind::Checkbox { equals: true },
                }],
            )]),
        };

        let query = DatabaseQuery {
            start_cursor: None,
            page_size: None,
            sorts: None,
            filter: Some(filter),
        };

        let pages = self
            .client
            .query_database(DatabaseKind::Source, Some(query))
            .await?
            .results;

        Ok(map_page_to_links(pages))
    }

    pub async fn get_feed_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut pages = vec![];
        let mut cursor: Option<String> = None;

        loop {
            let query = DatabaseQuery {
                start_cursor: match cursor {
                    Some(ref c) => Some(c.to_string()),
                    None => None,
                },
                page_size: None, // use default value (100)
                filter: None,
                sorts: None,
            };

            let current_pages = self
                .client
                .query_database(DatabaseKind::Feed, Some(query))
                .await?;

            pages.extend(current_pages.results);
            cursor = current_pages.next_cursor;

            if current_pages.has_more == false {
                break;
            }
        }

        Ok(map_page_to_links(pages))
    }

    pub async fn add_reader_entry(
        &self,
        title: String,
        link: String,
    ) -> Result<Page, Box<dyn Error>> {
        let page_props = HashMap::from([
            (
                "Title".to_string(),
                PropertyValue::Title {
                    title: vec![RichText::Text {
                        rich_text: None,
                        text: Text {
                            content: title,
                            link: None,
                        },
                    }],
                },
            ),
            ("Link".to_string(), PropertyValue::Url { url: Some(link) }),
            (
                "Read".to_string(),
                PropertyValue::Checkbox { checkbox: false },
            ),
            (
                "Starred".to_string(),
                PropertyValue::Checkbox { checkbox: false },
            ),
        ]);

        Ok(self
            .client
            .create_page(DatabaseKind::Feed, page_props)
            .await?)
    }

    pub async fn get_rss_items(source: String) -> Result<Channel, Box<dyn Error>> {
        let content = reqwest::get(source).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;
        Ok(channel)
    }
}

fn map_page_to_links(pages: Vec<Page>) -> Vec<String> {
    pages
        .iter()
        .map(|item| {
            let properties = item.properties.as_ref()?;

            match properties.get("Link") {
                Some(PropertyValue::Url { url }) => match url {
                    Some(url) => Some(url.to_string()),
                    None => None,
                },

                _ => None,
            }
        })
        .filter_map(identity)
        .collect::<Vec<String>>()
}
