use std::env::Args;

use actix_web::{get, App, HttpServer, Responder};

use error_chain::error_chain;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(serve_feed))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/feed")]
async fn serve_feed() -> impl Responder {
    let upstream_feed_url: String = "".to_string();

    match download_feed(upstream_feed_url).await {
        Ok(feed_contents) => format!("{}", feed_contents),
        Err(e) => format!("Failed to fetch feed with error: {:?}", e),
    }
}

async fn download_feed(upstream_feed_url: String) -> reqwest::Result<String> {
    let feed_contents = reqwest::get(upstream_feed_url).await?.text().await?;
    Ok(feed_contents)
}
