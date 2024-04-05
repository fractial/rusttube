use std::error::Error;
use std::fs::File;
use std::io::Write;
use serde_json::{from_str, Value};
use crate::request::{get_request, ResponseType::Stream};

mod request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let id = "XZogcY3aEsg";
    let url = format!("https://pipedapi.kavin.rocks/streams/{}", id);

    let result = get_request(&url, None).await?;

    let body = String::from_utf8_lossy(&result).to_string();

    let parsed: Value = from_str(&body).expect("Failed to parse JSON");
    let title = parsed["title"].as_str().unwrap_or("default");
    let stream_url = parsed["audioStreams"][3]["url"].as_str().unwrap_or("default");

    let mut file = File::create("out.mp4").expect("Error creating file");

    let stream = get_request(&stream_url, Some(Stream)).await?;
    file.write_all(&stream)?;
    Ok(())
}
