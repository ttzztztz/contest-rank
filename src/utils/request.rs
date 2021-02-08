use reqwest::{Method, Request, StatusCode};
use serde::{de::DeserializeOwned};
use std::io;

const MAX_RETRY_COUNT: u32 = 3;

pub async fn send_request<T>(url: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    for _ in 1u32..=MAX_RETRY_COUNT {
        let client = reqwest::Client::new();
        let request = Request::new(Method::GET, url.parse().unwrap());

        let resp = client.execute(request).await?;
        let status: StatusCode = resp.status();
        if status.as_u16() >= 400 {
            let resp_text = resp.text().await?;
            println!("[ERROR] status_code={}, {}", status, resp_text);
        } else {
            let response_obj = resp.json::<T>().await?;
            return Ok(response_obj);
        }
    }

    return Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "Error request after max retry count",
    )));
}
