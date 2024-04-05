use std::error::Error;
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT}};
use futures::stream::{self, StreamExt};
use std::time::Instant;

pub enum ResponseType {
    Default,
    Stream,
}

pub async fn get_request(url: &str, response_type: Option<ResponseType>) -> Result<Vec<u8>, Box<dyn Error>> {
    let now = Instant::now();

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("request"));
    let response = client
        .get(url)
        .headers(headers)
        .send().await?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("Request failed with `status`: {status}").into());
    }

    match response_type.unwrap_or(ResponseType::Default) {
        ResponseType::Default => {
            let body = response.text().await?;

            let elapsed = now.elapsed();
            println!("{:?}", elapsed);

            Ok(body.into_bytes())
        }
        ResponseType::Stream => {
            let mut data = Vec::new();

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                data.extend_from_slice(&chunk);
            }

            let elapsed = now.elapsed();
            println!("{:?}", elapsed);

            Ok(data)
        }
    }
}