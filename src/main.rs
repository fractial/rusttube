use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::string::ToString;
use std::time::Instant;
use serde_json::{from_str, Value};
use request::get_request;
use tokio;

mod request;

const URL: &str = "https://piped-api.lunar.icu/";

async fn download(id: &str) -> Result<(), Box<dyn Error>> {
    let time = Instant::now();

    let mut url = String::from(URL);
    url.push_str(format!("streams/{}", id).as_str());
    print!("{} ", url);

    let response = get_request(&url).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;

    let title = data["title"].to_string().replace("\"", "").replace(" ", "");
    let audio_stream = data["audioStreams"][3]["url"].to_string().replace("\"", "");
    let stream = get_request(&audio_stream).await?.bytes().await?.to_vec();

    let mut dir = env::var("USERPROFILE")?;
    dir.push_str("/Music");
    let file_type = "mp4";
    let file_path = format!("{}/{}.{}", dir, title, file_type);
    let mut file = File::create(file_path).expect("Error creating file");
    file.write_all(&stream)?;

    println!("{:?}", time.elapsed());

    Ok(())
}

async fn download_playlist(id: &str) -> Result<(), Box<dyn Error>> {
    let mut url = String::from(URL);
    url.push_str(format!("playlists/{}", id).as_str());
    println!("{}", url);

    let response = get_request(&url).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;
    let streams = data["relatedStreams"].as_array().expect("");

    for stream in streams {
        let id = stream["url"].to_string().replace("\"", "").replace("/watch?v=", "");
        download(&id).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let id = &args[1];

    if args.iter().any(|arg| arg == "--playlist" || arg == "-p") {
        download_playlist(id).await?;
        return Ok(());
    }

    download(id).await?;

    Ok(())
}