use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use serde_json::{from_str, Value};
use request::get_request;
use url::Url;

mod request;

const URL: &str = "https://pipedapi.kavin.rocks";

fn get_file_name(file_name: &str) -> String {
    let pattern = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    pattern.replace_all(&file_name, "").to_string()
}

async fn download(url: &Url, id: &str) -> Result<(), Box<dyn Error>> {
    let time = Instant::now();

    url.join(format!("streams/{}", id).as_str())?;
    let response = get_request(url.as_str()).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;

    let title = data["title"].to_string();
    let file_name = get_file_name(&title);

    let audio_stream = data["audioStreams"][3]["url"].to_string().replace("\"", "");
    let stream = get_request(&audio_stream).await?.bytes().await?.to_vec();

    let mut dir = env::var("USERPROFILE")?;
    dir.push_str("/Music");
    let file_type = "mp3";
    let file_path = format!("{}/{}.{}", dir, file_name, file_type);
    let mut file = File::create(file_path).expect("Error creating file");
    file.write_all(&stream)?;

    println!("{url} `{file_name}` {:?}", time.elapsed());

    Ok(())
}

async fn download_playlist(url: &Url, id: &str) -> Result<(), Box<dyn Error>> {
    url.join(format!("playlists/{}", id).as_str())?;
    println!("{}", url);

    let response = get_request(url.as_str()).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;
    let streams = data["relatedStreams"].as_array().expect("");

    for stream in streams {
        let id = stream["url"].to_string().replace("\"", "").replace("/watch?v=", "");
        download(url, &id).await?;
    }

    Ok(())
}

async fn search(url: &Url, search_query: &str) -> Result<(), Box<dyn Error>> {
    url.join(format!("search?q=\"{}\"&filter=all", search_query).as_str())?;

    let response = get_request(url.as_str()).await?;
    let body = response.text().await?;
    let data: Value = from_str(&body)?;
    let items = data["items"].as_array().expect("");

    for (index, item) in items.iter().enumerate() {
        let title = item["title"].to_string();
        println!("[{}] {}", index, title);
    }

    let index = read_index()?;
    let id = items[index]["url"].to_string().replace("\"", "").replace("/watch?v=", "");
    download(url, &id).await?;

    Ok(())
}

fn read_index() -> Result<usize, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let index: usize = input.trim().parse()?;

    Ok(index)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut url = Url::parse(URL)?;
    let args: Vec<String> = env::args().collect();
    let id = &args[1];

    if let Some(index) = args.iter().position(|arg| arg == "--instance" || arg == "-i") {
        let url_instance = &args.get(index + 1).ok_or("");
        if !url_instance.is_empty() {
            url = Url::parse(url_instance)?;
        }
    }

    if let Some(index) = args.iter().position(|arg| arg == "--search" || arg == "-s") {
        let search_query = &args[index + 1];
        if !search_query.is_empty() {
            search(&url, search_query).await?;
            return Ok(());
        }
    }

    if args.iter().any(|arg| arg == "--playlist" || arg == "-p") {
        download_playlist(&url, id).await?;
        return Ok(());
    }

    if let Some(id) = args.get(1) {
        download(&url, id).await?;
    }

    Ok(())
}