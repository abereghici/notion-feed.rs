use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use futures::{future, try_join};
use rss::{Channel, Item};

use crate::notion::{
    database::{DatabaseFilter, DatabaseKind, DatabaseQuery, Filter, FilterKind},
    models::{Date, Page, PropertyValue, RichText, Text},
    Client,
};
use std::{collections::HashMap, convert::identity, error::Error};

use super::{feed_item::FeedItem, source::Source};

pub struct Feed<'a> {
    client: &'a Client<'a>,
}

impl<'a> Feed<'a> {
    pub fn new(client: &'a Client<'a>) -> Feed<'a> {
        Self { client }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let (source_list, feed_list) = try_join!(self.get_source_list(), self.get_feed_list())?;

        let channel_items = future::join_all(
            source_list
                .into_iter()
                .map(|source| Feed::<'_>::get_rss_items(source)),
        )
        .await;

        let mut all_items = vec![];

        channel_items.iter().for_each(|item| {
            if let Ok(item) = item {
                all_items.extend(item);
            }
        });

        let feed_list_links = feed_list
            .into_iter()
            .map(|item| item.link)
            .collect::<Vec<String>>();

        future::join_all(
            all_items
                .iter()
                .filter(|item| match &item.link {
                    Some(link) => !feed_list_links.contains(link),
                    None => false,
                })
                .map(|item| {
                    let title = &item.title;
                    let link = &item.link;
                    let pub_date = &item.pub_date;

                    let created_date = match pub_date {
                        Some(pub_date) => {
                            let pub_date =
                                parse_date(&pub_date).unwrap_or(Local::now().date_naive());
                            Some(Utc.from_utc_date(&pub_date).and_hms(0, 0, 0))
                        }
                        None => Some(Utc::now()),
                    };

                    if let (Some(title), Some(link), Some(created_date)) =
                        (title, link, created_date)
                    {
                        return Some(self.add_feed_entry(
                            title.to_string(),
                            link.to_string(),
                            created_date,
                        ));
                    }
                    None
                })
                .filter_map(identity),
        )
        .await;

        Ok(())
    }

    pub async fn get_source_list(&self) -> Result<Vec<Source>, Box<dyn Error>> {
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

        return Ok(pages
            .iter()
            .map(|page| return Source::new(page))
            .filter_map(identity)
            .collect::<Vec<Source>>());
    }

    pub async fn get_feed_list(&self) -> Result<Vec<FeedItem>, Box<dyn Error>> {
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

        return Ok(pages
            .iter()
            .map(|page| return FeedItem::new(page))
            .filter_map(identity)
            .collect::<Vec<FeedItem>>());
    }

    pub async fn add_feed_entry(
        &self,
        title: String,
        link: String,
        created_time: DateTime<Utc>,
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
            (
                "Published At".to_string(),
                PropertyValue::Date {
                    date: Some(Date {
                        start: Some(created_time),
                        end: None,
                    }),
                },
            ),
        ]);

        let result = self
            .client
            .create_page(DatabaseKind::Feed, page_props)
            .await;

        Ok(result?)
    }

    pub async fn get_rss_items(source: Source) -> Result<Vec<Item>, Box<dyn Error>> {
        let content = reqwest::get(&source.link).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;

        let offset_date = source.offset_date;

        if let Some(offset_date) = offset_date {
            let items = channel
                .items
                .into_iter()
                .filter(|item| {
                    let pub_date = item.pub_date.as_ref();

                    if let Some(pub_date) = pub_date {
                        let pub_date = parse_date(pub_date);

                        if let Some(pub_date) = pub_date {
                            return pub_date.ge(&offset_date);
                        }

                        return false;
                    }

                    return true;
                })
                .collect();

            return Ok(items);
        }

        Ok(channel.items)
    }
}

fn parse_date(input: &String) -> Option<NaiveDate> {
    let rfc2822 = DateTime::parse_from_rfc2822(&input);

    if let Ok(rfc2822) = rfc2822 {
        return Some(rfc2822.date_naive());
    }

    let rfc3339 = DateTime::parse_from_rfc3339(&input);

    if let Ok(rfc3339) = rfc3339 {
        return Some(rfc3339.date_naive());
    }

    let date_only = NaiveDate::parse_from_str(&input, "%Y-%m-%d");
    if let Ok(date_only) = date_only {
        return Some(date_only);
    }

    None
}
