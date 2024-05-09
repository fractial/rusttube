use std::env;
use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use serde_json::{from_str, Value};
use url::Url;
use request::get_request;

mod request;

const URL: &str = "https://pipedapi.kavin.rocks";
const DIR: &str = "USERPROFILE";
const TYPE: &str = "mp3";

async fn download(url: &Url, id: &str) -> Result<(), Box<dyn Error>> {
    let time = Instant::now();

    let mut url = url.clone();
    let stream_url = format!("streams/{}", &id);
    url = url.join(&stream_url).unwrap();
    let res = get_request(url.as_str()).await?.text().await?;
    let data: Value = from_str(&res)?;

    let name = if let Some(title) = data.get("title") {
        title.to_string()
    } else {
        "unknown".to_string()
    };
    let file_name = get_file_name(&name);
    let audio_stream = if let Some(audio_stream) = data["audioStreams"][3]["url"].as_str() {
        audio_stream.to_string()
    } else {
        panic!("`audio_stream` doesn't exist")
    };
    let stream = get_request(&audio_stream).await?.bytes().await?.to_vec();
    write_file(&file_name, &stream)?;

    println!("{} {} {:?}", url, file_name, time.elapsed());

    Ok(())
}

fn write_file(file_name: &str, stream: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let dir = Path::new(&env::var(DIR)?).join("Music");
    let file_path = dir.join(format!("{}.{}", file_name, TYPE));
    let mut file = std::fs::File::create(file_path).expect("failed creating the file");
    file.write_all(&stream)?;

    Ok(())
}

async fn download_playlist(url: &Url, id: &str) -> Result<(), Box<dyn Error>> {
    let mut clone_url = url.clone();
    let playlist_url = format!("playlists/{}", &id);
    clone_url = clone_url.join(&playlist_url).unwrap();
    let res = get_request(clone_url.as_str()).await?.text().await?;
    let data: Value = from_str(&res)?;
    let related_streams = data["relatedStreams"].as_array().expect("failed reading `related_streams`");

    for stream in related_streams {
        let mut id = stream["url"].to_string().replace("/watch?v=", "");
        id = get_file_name(&id);
        download(&url, &id).await?;
    }

    Ok(())
}

async fn search(url: &Url, search_query: &str) -> Result<(), Box<dyn Error>> {
    let mut clone_url = url.clone();
    let search_url = format!("search?q=\"{}\"&filter=all", search_query);
    clone_url = clone_url.join(&search_url)?;
    let res = get_request(clone_url.as_str()).await?.text().await?;
    let data: Value = from_str(&res)?;
    let items = data["items"].as_array().expect("failed reading `items`");

    for (index, item) in items.iter().enumerate() {
        let name = item["title"].to_string();
        println!("[{}] {}", index, name);
    }

    let index = read_line()?;
    let mut id = items[index]["url"].to_string().replace("/watch?v=", "");
    id = get_file_name(&id);
    download(&url, &id).await?;

    Ok(())
}

fn read_line() -> Result<usize, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let index = input.trim().parse()?;

    Ok(index)
}

fn get_file_name(file_name: &str) -> String {
    let pattern = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();

    pattern.replace_all(&file_name, "").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args:Vec<String> = env::args().collect();
    let mut url = Url::parse(URL).expect("parsing url failed");
    let mut search_query = &String::new();
    let mut playlist = false;

    for (index, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--instance" | "-i" => {
                if let Some(value) = args.get(index + 1) {
                    if let Ok(instance_url) = Url::parse(value) {
                        url = instance_url;
                    }
                }
            }
            "--search" | "-s" => {
                if let Some(value) = args.get(index + 1) {
                    search_query = value;
                }
            }
            "--playlist" | "-p" => {
                playlist = true;
            }
            _ => {}
        }
    }

    if let Some(id) = args.get(1) {
        if !search_query.is_empty() {
            search(&url, search_query).await.expect("search failed");
            return Ok(());
        }

        if playlist {
            download_playlist(&url, id).await.expect("playlist download failed");
            return Ok(());
        }

        download(&url, id).await.expect("download failed");
    }

    Ok(())
}