use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use serde_json::{from_str, Value};
use crate::request::{get_request, ResponseType::Stream};

mod request;

async fn download(id: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://piped-api.lunar.icu/streams/{}", id);
    println!("{}", url);

    let result = get_request(&url, None).await?;
    let body = String::from_utf8_lossy(&result).to_string();

    let parsed: Value = from_str(&body).expect("Failed to parse JSON");
    let title = parsed["title"].as_str().unwrap_or("default");
    let name = title.replace(" ", "");
    let stream_url = parsed["audioStreams"][3]["url"].as_str().unwrap_or("default");

    let mut download_dir = env::var("USERPROFILE").expect("Failed to find `HOME` directory");
    download_dir.push_str("/Music");
    let file_path = format!("{}/{}.mp4", download_dir, name);
    let mut file = File::create(file_path).expect("Error creating file");

    let stream = get_request(&stream_url, Some(Stream)).await?;
    file.write_all(&stream)?;

    Ok(())
}

async fn playlist(id: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://piped-api.lunar.icu/playlists/{}", id);
    println!("{}", url);

    let result = get_request(&url, None).await?;
    let body = String::from_utf8_lossy(&result).to_string();

    let parsed: Value = from_str(&body).expect("Failed to parse JSON");
    let streams = parsed["relatedStreams"].as_array().expect("Failed to parse `relatedStreams`");

    for stream in streams {
        let stream_uri = stream["url"].to_string();
        let stream_url = stream_uri.trim_matches('"').to_string();
        let stream_id = stream_url.replace("/watch?v=", "");
        let _ = download(&stream_id).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let id = &args[1];

    if args.contains(&String::from("--playlist")) || args.contains(&String::from("-p")) {
        let _ = playlist(id).await;

        return Ok(());
    }

    let _ = download(&id).await;

    Ok(())
}
