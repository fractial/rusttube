use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::string::ToString;
use std::time::Instant;
use regex::Regex;
use serde_json::{from_str, Value};
use request::get_request;
use tokio;

mod request;

fn get_file_name(file_name: &str) -> String {
    let pattern = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    pattern.replace_all(&file_name, "").to_string()
}

async fn download(url_instance: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let time = Instant::now();

    let mut url = String::from(url_instance);
    url.push_str(format!("/streams/{}", id).as_str());
    print!("{} ", url);

    let response = get_request(&url).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;

    let title = data["title"].to_string();
    let file_name = get_file_name(&title);

    print!("{} ", file_name);
    let audio_stream = data["audioStreams"][3]["url"].to_string().replace("\"", "");
    let stream = get_request(&audio_stream).await?.bytes().await?.to_vec();

    let mut dir = env::var("USERPROFILE")?;
    dir.push_str("/Music");
    let file_type = "mp4";
    let file_path = format!("{}/{}.{}", dir, file_name, file_type);
    let mut file = File::create(file_path).expect("Error creating file");
    file.write_all(&stream)?;

    println!("{:?}", time.elapsed());

    Ok(())
}

async fn download_playlist(url_instance: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let mut url = String::from(url_instance);
    url.push_str(format!("/playlists/{}", id).as_str());
    println!("{}", url);

    let response = get_request(&url).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;
    let streams = data["relatedStreams"].as_array().expect("");

    for stream in streams {
        let id = stream["url"].to_string().replace("\"", "").replace("/watch?v=", "");
        download(&url, &id).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut url = "https://pipedapi.kavin.rocks";
    let args: Vec<String> = env::args().collect();
    let id = &args[1];

    if let Some(index) = args.iter().position(|arg| arg == "--instance" || arg == "-i") {
        let url_instance = &args[index + 1];
        if !url_instance.is_empty() {
            url = url_instance;
        }
    }

    if args.iter().any(|arg| arg == "--playlist" || arg == "-p") {
        download_playlist(url, id).await?;
        return Ok(());
    }

    download(url, id).await?;

    Ok(())
}