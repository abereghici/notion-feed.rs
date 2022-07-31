use clap::Parser;
use config::Config;
use feed::Feed;
use notion::Client;
use std::{error::Error, process};

mod config;
mod feed;
mod notion;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Arguments {
    #[clap(short, long)]
    notion_source_database_id: Option<String>,
    #[clap(short, long)]
    notion_feed_database_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();

    let config = Config::new(args.notion_source_database_id, args.notion_feed_database_id)
        .unwrap_or_else(|err| {
            eprintln!("Failed to create application config: {}", err);
            process::exit(1)
        });

    let notion_client = Client::new(&config).unwrap_or_else(|err| {
        eprintln!("{}", format!("Failed to create the notion client: {}", err));
        process::exit(1)
    });

    Feed::new(&notion_client).run().await.unwrap_or_else(|err| {
        eprintln!(
            "{}",
            format!("An error has occurred while processing data: {}", err)
        );
        process::exit(1)
    });

    process::exit(0)
}
