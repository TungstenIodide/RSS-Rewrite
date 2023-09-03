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

// For debugging
// use std::fs::File;
// use std::io::Write;

lazy_static! {
    static ref CONFIGS: Vec<FeedConfig> = read_configuration();
}

#[derive(Serialize, Deserialize)]
struct Replace {
    #[serde(with = "serde_regex")]
    match_pattern: Regex,
    replace_with: String,
}

#[derive(Serialize, Deserialize)]
struct FeedConfig {
    name: String,
    url: String,
    replace_rules: Vec<Replace>,
}

fn read_configuration() -> Vec<FeedConfig> {
    let configuration = match fs::read_to_string("./feeds.json") {
        Ok(x) => x,
        Err(e) => panic!("Failed to read ./feeds.json file with error: {}", e),
    };

    let feeds_configurations: Vec<FeedConfig> = match serde_json::from_str(&configuration) {
        Ok(x) => x,
        Err(e) => panic!("Failed to parse JSON with error: {}", e),
    };

    feeds_configurations
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
    let feed_config: &FeedConfig = match get_feed_config(feed.to_string()) {
        Ok(x) => x,
        Err(feed_name) => return not_found(format!("Feed not found: {}", feed_name)).await,
    };

    let original_feed: String = match download_feed(&feed_config.url).await {
        Ok(feed_contents) => feed_contents,
        Err(e) => return not_found(format!("Failed to fetch feed with error: {}", e)).await,
    };

    let feed_content = feed_modifier(&feed_config, original_feed);

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/rss+xml")
        .body(feed_content))
}

fn get_feed_config(feed_name: String) -> Result<&'static FeedConfig, String> {
    for config in CONFIGS.iter() {
        if feed_name == config.name {
            return Ok(config);
        }
    }
    return Err(feed_name);
}

fn feed_modifier(feed_config: &FeedConfig, original_feed: String) -> String {
    let mut content = original_feed;

    for replace_rule in feed_config.replace_rules.iter() {
        content = replace_rule
            .match_pattern
            .replace_all(&mut content, &replace_rule.replace_with)
            .to_string();
    }

    // For debugging
    //    let mut output = File::create("./feed.rss").expect("a");
    //    write!(output, "{}", content).expect("a");

    return content.to_string();
}

// TODO: Improve the error handling
async fn download_feed(upstream_feed_url: &String) -> reqwest::Result<String> {
    Ok(reqwest::get(upstream_feed_url).await?.text().await?)
}

async fn not_found(error_message: String) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(error_message))
}
