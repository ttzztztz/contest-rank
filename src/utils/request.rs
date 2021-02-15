use serde::de::DeserializeOwned;
use std::io;

const MAX_RETRY_COUNT: u32 = 3;
const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.146 Safari/537.36";

async fn _send_one_request<T>(url: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let resp = client.get(url).send().await?;
    return Ok(resp.json::<T>().await?);
}

pub async fn send_request<T>(url: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    for retry_idx in 1u32..=MAX_RETRY_COUNT {
        match _send_one_request::<T>(url).await {
            Ok(resp) => return Ok(resp),

            Err(err) => {
                println!(
                    "[Error] When send_request, retry_idx=#{}, e={}",
                    retry_idx, err
                );
            }
        }
    }

    return Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "Error request after max retry count",
    )));
}
