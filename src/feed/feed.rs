use futures::{future, try_join};
use rss::Channel;

use crate::{
    feed::source,
    notion::{
        database::{DatabaseFilter, DatabaseKind, DatabaseQuery, Filter, FilterKind},
        Client,
    },
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

        let channels = future::join_all(
            source_list
                .into_iter()
                .map(|source| Feed::<'_>::get_rss_items(source)),
        )
        .await;

        dbg!(channels);

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

    pub async fn get_rss_items(source: Source) -> Result<Channel, Box<dyn Error>> {
        let content = reqwest::get(&source.link).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;

        Ok(channel)
    }
}
