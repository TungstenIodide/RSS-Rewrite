use actix_web::{get, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Serialize, Deserialize)]
struct FeedConfig {
    name: String,
    url: String,
    match_pattern: String,
    replace_pattern: String,
}

fn get_feed_config(feed_name: String) -> Result<FeedConfig, String> {
    let config = fs::read_to_string("./feeds.json").expect("Unable to read file!");
    let feed_config: FeedConfig = serde_json::from_str(&config).expect("Couldn't parse JSON");

    // TODO: Support for multiple feeds
    if feed_name == feed_config.name {
        Ok(feed_config)
    } else {
        Err(format!("Config not found for feed {}", feed_name))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(rss_exp))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

// TODO: Return a file instead of text
// TODO: Modify RSS address to address from GET request
#[get("/{feed}")]
async fn rss_exp(feed: web::Path<String>) -> impl Responder {
    let feed_config: FeedConfig = match get_feed_config(feed.to_string()) {
        Ok(config) => config,
        Err(e) => return format!("{}", e),
    };

    let feed_content: String = feed_modifier(feed_config).await;

    format!("{}", feed_content)
}

// TODO: actually modify stuff
async fn feed_modifier(feed_config: FeedConfig) -> String {
    let feed = match download_feed(feed_config.url).await {
        Ok(feed_contents) => feed_contents,
        Err(e) => panic!("Failed to fetch feed with error: {}", e),
    };

    feed
}

async fn download_feed(upstream_feed_url: String) -> reqwest::Result<String> {
    let feed_contents = reqwest::get(upstream_feed_url).await?.text().await?;
    Ok(feed_contents)
}
