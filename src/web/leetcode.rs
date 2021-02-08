use crate::model::{config::Settings, render::Submission, renderable::Renderable, website::{WebsiteContest, WebsiteUser}};
use chrono::{prelude::Utc, TimeZone};
use reqwest::{Method, Request, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use std::collections::HashSet;
use std::io;
use tokio::runtime::Runtime;

#[derive(Deserialize)]
struct SubmissionItem {
    fail_count: u32,
    date: u64,
    question_id: u32,
    submission_id: u32,
}

#[derive(Deserialize)]
struct RankItem {
    country_name: String,
    finish_time: i64,
    rank: u32,
    score: u32,
    username: String,
    data_region: String,
}
#[derive(Deserialize)]
struct LeetcodeRankRequest {
    submissions: Vec<SubmissionItem>,
    user_num: u64,
    is_past: bool,
    total_rank: Vec<RankItem>,
}

#[derive(Deserialize)]
struct LeetcodeContestInfo {
    start_time: i64,
    title: String,
}

#[derive(Deserialize)]
struct LeetcodeContestInfoRequest {
    contest: LeetcodeContestInfo,
}

const MAX_RETRY_COUNT: u32 = 3;

async fn send_request<T>(url: String) -> Result<T, Box<dyn std::error::Error>>
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

async fn send_contest_info_request(
    contest_type: &str,
    contest_id: u32,
) -> Result<LeetcodeContestInfoRequest, Box<dyn std::error::Error>> {
    let contest_type_full: &str;
    if contest_type == "w" {
        contest_type_full = "weekly";
    } else {
        // contest_type == "b"
        contest_type_full = "biweekly";
    }

    let url = format!(
        "https://leetcode.com/contest/api/info/{contest_type}-contest-{id}/",
        id = contest_id,
        contest_type = contest_type_full
    );
    let res = send_request::<LeetcodeContestInfoRequest>(url).await?;
    return Ok(res);
}

async fn send_contest_rank_request(
    contest_type: &str,
    contest_id: u32,
    page: u32,
) -> Result<LeetcodeRankRequest, Box<dyn std::error::Error>> {
    let contest_type_full: &str;
    if contest_type == "w" {
        contest_type_full = "weekly";
    } else {
        // contest_type == "b"
        contest_type_full = "biweekly";
    }

    let url = format!(
        "https://leetcode.com/contest/api/ranking/{contest_type}-contest-{id}?pagination={page}&region=global",
        id = contest_id,
        contest_type = contest_type_full,
        page = page
    );

    let res = send_request::<LeetcodeRankRequest>(url).await?;
    return Ok(res);
}

async fn request_leetcode(
    contest_type: String,
    contest_id: u32,
    players: Vec<String>,
) -> Result<WebsiteContest, Box<dyn std::error::Error>> {
    let contest_info = send_contest_info_request(&contest_type, contest_id)
        .await?
        .contest;

    let mut searching_players = HashSet::<String>::new();
    for player in players.iter() {
        searching_players.insert(player.clone());
    }

    let mut website_players = Vec::<WebsiteUser>::new();
    let mut page = 1u32;
    while !searching_players.is_empty() {
        let rank = send_contest_rank_request(&contest_type, contest_id, page).await?;
        assert_eq!(rank.submissions.len(), rank.total_rank.len());

        let n = rank.submissions.len();
        for i in 0..n {
            let submission = &rank.submissions[i];
            let rank = &rank.total_rank[i];

            if searching_players.contains(&rank.username) {
                searching_players.remove(&rank.username);
                let submissions_vec = Vec::<Submission>::new();

                website_players.push(WebsiteUser {
                    username: rank.username.clone(),
                    country: rank.country_name.clone(),
                    finish_time: rank.finish_time - contest_info.start_time,
                    global_rank: rank.rank,
                    score: rank.score,

                    submissions: submissions_vec,
                });
            }
        }
        page += 1u32;
    }

    return Ok(WebsiteContest {
        name: contest_info.title,
        date: Utc
            .timestamp(contest_info.start_time, 0)
            .format("%Y-%m-%d %a %H:%M")
            .to_string(),
        players: vec![],
    });
}

pub struct LeetcodeWeb {}

impl Renderable for LeetcodeWeb {
    fn render(settings: Settings) -> Vec<WebsiteContest> {
        let mut contests = Vec::<WebsiteContest>::new();

        for contest_id in settings.config.contests.iter() {
            if contest_id.is_empty() {
                continue;
            }

            if contest_id.starts_with("w") || contest_id.starts_with("b") {
                let contest_type = (&contest_id[0..1]).to_string();
                let contest_number = (&contest_id[1..]).parse::<u32>().unwrap();

                if settings.verbose {
                    println!(
                        "[LOG] Contest Type={}, number={}",
                        contest_type, contest_number
                    );
                }

                let runtime = Runtime::new().unwrap();
                match runtime.block_on(request_leetcode(
                    contest_type,
                    contest_number,
                    settings.config.users.to_vec(),
                )) {
                    Ok(resp) => contests.push(resp),
                    Err(e) => panic!("[Error] when fetching contest {}", e),
                }
            } else {
                panic!("contest_id={} invalid", contest_id)
            }
        }
        return contests;
    }

    fn website_name() -> String {
        return String::from("leetcode");
    }
}
