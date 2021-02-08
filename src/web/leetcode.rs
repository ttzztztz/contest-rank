use crate::utils::{finish_time, null, request};
use crate::{
    model::{
        config::LeetcodeConfig,
        render::{Submission, SubmissionStatus},
        renderable::Renderable,
        website::{WebsiteContest, WebsiteUser},
    },
    service::cache::{get_cache, set_cache},
};
use chrono::{prelude, TimeZone};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize)]
struct SubmissionItem {
    fail_count: u32,
    date: i64,
    question_id: u32,
    submission_id: u32,
}

#[derive(Serialize, Deserialize)]
struct RankItem {
    #[serde(deserialize_with = "null::parse_null_or_string")]
    country_name: String,

    finish_time: i64,
    rank: u32,
    score: u32,
    username: String,
}
#[derive(Serialize, Deserialize)]
struct LeetcodeRankRequest {
    submissions: Vec<HashMap<String, SubmissionItem>>,
    user_num: u64,
    is_past: bool,
    total_rank: Vec<RankItem>,
}

#[derive(Serialize, Deserialize)]
struct LeetcodeContestInfo {
    start_time: i64,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct LeetcodeQuestionInfo {
    credit: u32,
    id: u32,
    question_id: u32,
    title: String,
    title_slug: String,
}

#[derive(Serialize, Deserialize)]
struct LeetcodeContestInfoRequest {
    contest: LeetcodeContestInfo,
    questions: Vec<LeetcodeQuestionInfo>,
}

pub struct LeetcodeWeb {
    pub verbose: bool,
    pub config: LeetcodeConfig,
}

impl LeetcodeWeb {
    async fn send_contest_info_request(
        &self,
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
        let res = request::send_request::<LeetcodeContestInfoRequest>(&url).await?;
        return Ok(res);
    }

    async fn send_contest_rank_request(
        &self,
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

        if self.config.cache {
            if let Some(memo) = get_cache::<LeetcodeRankRequest>(&url) {
                if self.verbose {
                    println!("[INFO] cache hit request url={}", url);
                }
                return Ok(memo);
            }
        }

        let res = request::send_request::<LeetcodeRankRequest>(&url).await?;
        if self.config.cache && res.is_past {
            if self.verbose {
                println!("[INFO] cache set url={}", url);
            }
            set_cache(&url, &res);
        }
        return Ok(res);
    }

    async fn request_leetcode(
        &self,
        contest_type: String,
        contest_id: u32,
        players: Vec<String>,
    ) -> Result<WebsiteContest, Box<dyn std::error::Error>> {
        let contest_req = self
            .send_contest_info_request(&contest_type, contest_id)
            .await?;

        let contest_info = contest_req.contest;
        let questions = contest_req.questions;

        let mut searching_players = HashSet::<String>::new();
        for player in players.iter() {
            searching_players.insert(player.clone());
        }

        let mut website_players = Vec::<WebsiteUser>::new();
        let mut page = 1u32;
        while !searching_players.is_empty() && (page + 1u32) * 25u32 < self.config.max_rank {
            if self.verbose {
                println!("[INFO] ({}), current page={}", contest_info.title, page);
            }

            let rank = self
                .send_contest_rank_request(&contest_type, contest_id, page)
                .await?;
            assert_eq!(rank.submissions.len(), rank.total_rank.len());

            let playeres_in_page = rank.submissions.len();
            for i in 0..playeres_in_page {
                let submission_hashmap = &rank.submissions[i];
                let rank = &rank.total_rank[i];

                if !searching_players.contains(&rank.username) {
                    continue;
                }

                searching_players.remove(&rank.username);
                let mut submissions_vec = Vec::<Submission>::new();

                for question_index in 0..questions.len() {
                    let question = &questions[question_index];
                    let question_id = question.question_id;
                    let question_id_str = question_id.to_string();

                    match submission_hashmap.get(&question_id_str) {
                        None => {
                            submissions_vec.push(Submission {
                                fail_count: 0,
                                finish_time: String::from(""),
                                status: SubmissionStatus::Unaccepted,
                                score: 0,
                                title: format!("T{}", question_index + 1),
                            });
                        }
                        Some(submission) => {
                            submissions_vec.push(Submission {
                                fail_count: submission.fail_count,
                                finish_time: finish_time::seconds_to_finish_time(
                                    submission.date - contest_info.start_time,
                                ),
                                status: SubmissionStatus::Accepted,
                                score: question.credit,
                                title: format!("T{}", question_index + 1),
                            });
                        }
                    }
                }

                website_players.push(WebsiteUser {
                    username: rank.username.clone(),
                    country: rank.country_name.clone(),
                    finish_time: rank.finish_time - contest_info.start_time,
                    global_rank: rank.rank,
                    score: rank.score,
                    submissions: submissions_vec,
                });
            }

            page += 1u32;
        }

        return Ok(WebsiteContest {
            name: String::from("LeetCode ") + &contest_info.title,
            date: prelude::Local
                .timestamp(contest_info.start_time, 0)
                .format_localized("%Y-%m-%d %a %H:%M", prelude::Locale::ja_JP)
                .to_string(),
            players: website_players,
        });
    }
}

impl Renderable for LeetcodeWeb {
    fn render(&self, contests: &Vec<String>, users: &Vec<String>) -> Vec<WebsiteContest> {
        let verbose = false;

        let mut web_contests = Vec::<WebsiteContest>::new();
        for contest_id in contests.iter() {
            if contest_id.is_empty() {
                continue;
            }

            if contest_id.starts_with("w") || contest_id.starts_with("b") {
                let contest_type = (&contest_id[0..1]).to_string();
                let contest_number = (&contest_id[1..]).parse::<u32>().unwrap();

                if verbose {
                    println!(
                        "[LOG] Contest Type={}, number={}",
                        contest_type, contest_number
                    );
                }

                let runtime = Runtime::new().unwrap();
                match runtime.block_on(self.request_leetcode(
                    contest_type,
                    contest_number,
                    users.to_vec(),
                )) {
                    Ok(resp) => web_contests.push(resp),
                    Err(e) => panic!("[Error] when fetching contest {}", e),
                }
            } else {
                panic!("contest_id={} invalid", contest_id)
            }
        }
        return web_contests;
    }

    fn website_name(&self) -> String {
        return String::from("leetcode");
    }

    fn render_config(&self) -> Vec<WebsiteContest> {
        let config = &self.config;
        return self.render(&config.contests, &config.users);
    }
}
