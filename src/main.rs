#[macro_use]
extern crate serde_derive;
extern crate lazy_static;

extern crate serde;
extern crate serde_json;
extern crate serde_regex;

use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use std::fs;

#[derive(Serialize, Deserialize)]
struct FeedConfig {
    name: String,
    url: String,
    #[serde(with = "serde_regex")]
    match_pattern: Regex,
    replace_pattern: String,
}

fn read_configuration() -> FeedConfig {
    let configuration = match fs::read_to_string("./feeds.json") {
        Ok(x) => x,
        Err(e) => panic!("Failed to read ./feeds.json file with error: {}", e),
    };

    let feed_config: FeedConfig = match serde_json::from_str(&configuration) {
        Ok(x) => x,
        Err(e) => panic!("Failed to parse JSON with error: {}", e),
    };

    feed_config
}

lazy_static! {
    static ref CONFIG: FeedConfig = read_configuration();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(rss_rewrite))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

#[get("/{feed}")]
async fn rss_rewrite(feed: web::Path<String>) -> Result<HttpResponse, Error> {
    // TODO: Multiple feeds
    let feed_config: &FeedConfig = match get_feed_config(feed.to_string()) {
        Ok(()) => &CONFIG,
        Err(feed_name) => return not_found(format!("Feed not found: {}", feed_name)).await,
    };

    let mut feed_content: String = match download_feed(&feed_config.url).await {
        Ok(feed_contents) => feed_contents,
        Err(e) => return not_found(format!("Failed to fetch feed with error: {}", e)).await,
    };

    feed_content = feed_modifier(&feed_config, feed_content);

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/rss+xml")
        .body(feed_content))
}

// TODO: Support for multiple feeds
fn get_feed_config(feed_name: String) -> Result<(), String> {
    if feed_name == CONFIG.name {
        Ok(())
    } else {
        Err(feed_name)
    }
}

fn feed_modifier(feed_config: &FeedConfig, feed_content: String) -> String {
    feed_config
        .match_pattern
        .replace_all(&feed_content, &feed_config.replace_pattern)
        .to_string()
}

async fn download_feed(upstream_feed_url: &String) -> reqwest::Result<String> {
    Ok(reqwest::get(upstream_feed_url).await?.text().await?)
}

async fn not_found(error_message: String) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(error_message))
}
