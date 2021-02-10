use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::io;

const MAX_RETRY_COUNT: u32 = 3;
const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.146 Safari/537.36";

pub async fn send_request<T>(url: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    for retry_idx in 1u32..=MAX_RETRY_COUNT {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

        let resp = client.get(url).send().await?;
        let status: StatusCode = resp.status();
        if status.as_u16() >= 400 {
            let resp_text = resp.text().await?;
            println!(
                "[ERROR] request retry#{}, url={}, status_code={}, {}",
                retry_idx, url, status, resp_text
            );
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
