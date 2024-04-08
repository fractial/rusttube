
use std::error::Error;
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT}, Response};

pub async fn get_request(url: &str) -> Result<Response, Box<dyn Error>> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("request"));
    let response = client.get(url).headers(headers).send().await?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("Request failed with status: {}", status).into());
    };

    Ok(response)
}