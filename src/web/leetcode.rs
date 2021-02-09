use crate::model::{
    config::LeetcodeConfig,
    render::{Submission, SubmissionStatus},
    renderable::Renderable,
    website::{WebsiteContest, WebsiteUser},
};
use crate::service::cache::Cache;
use crate::utils::{finish_time, null, request};
use chrono::{prelude, TimeZone};
use futures::{executor, future};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

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
    pub cache: Arc<RwLock<Cache>>,

    pub enable_cache: RefCell<bool>,
}

const MAX_CONCURRENT_PAGE: u32 = 4u32;

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

        if *self.enable_cache.borrow() == true {
            let cloned_arc = self.cache.clone();
            let read_lock = cloned_arc.read().unwrap();
            if let Some(memo) = read_lock.get_cache::<LeetcodeRankRequest>(&url) {
                if self.verbose {
                    println!("[INFO] cache hit request url={}", url);
                }
                return Ok(memo);
            }
        }

        let res = request::send_request::<LeetcodeRankRequest>(&url).await?;
        if *self.enable_cache.borrow() == true && res.is_past {
            let cloned_arc = self.cache.clone();
            let mut write_lock = cloned_arc.write().unwrap();
            if self.verbose {
                println!("[INFO] cache set url={}", url);
            }
            write_lock.set_cache(&url, &res);
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
        while !searching_players.is_empty() && page * 25u32 < self.config.max_rank {
            let mut ranks = vec![];
            for page_offset in 0u32..MAX_CONCURRENT_PAGE {
                if self.verbose {
                    println!(
                        "[INFO] ({}), current page={}",
                        contest_info.title,
                        page + page_offset
                    );
                }
                ranks.push(self.send_contest_rank_request(
                    &contest_type,
                    contest_id,
                    page + page_offset,
                ))
            }
            let ranks = future::join_all(ranks).await;
            // todo: excceed page limit check

            for rank_result in ranks.iter() {
                if let Ok(rank) = rank_result {
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
                }
            }

            page += MAX_CONCURRENT_PAGE;
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

    async fn __render(&self, contests: &Vec<String>, users: &Vec<String>) -> Vec<WebsiteContest> {
        let verbose = false;

        let mut web_contests = Vec::<WebsiteContest>::new();
        let mut contest_futures = vec![];

        for contest_id in contests.iter() {
            if contest_id.is_empty() {
                continue;
            }

            if contest_id.starts_with("w") || contest_id.starts_with("b") {
                let contest_type = (&contest_id[0..1]).to_string();
                let contest_number = (&contest_id[1..]).parse::<u32>().unwrap();

                if verbose {
                    println!(
                        "[INFO] Contest Type={}, number={}",
                        contest_type, contest_number
                    );
                }

                contest_futures.push(self.request_leetcode(
                    contest_type,
                    contest_number,
                    users.to_vec(),
                ));
            } else {
                panic!("contest_id={} invalid", contest_id)
            }
        }

        let future = future::join_all(contest_futures).await;
        for future_result in future.iter() {
            match future_result {
                Ok(user) => {
                    web_contests.push(user.clone());
                }
                Err(e) => {
                    println!("[ERROR] when fetching contest {}", e);
                }
            }
        }
        return web_contests;
    }

    async fn render(&self, contests: &Vec<String>, users: &Vec<String>) -> Vec<WebsiteContest> {
        *self.enable_cache.borrow_mut() = self.config.cache;
        return self.__render(contests, users).await;
    }
}

impl Renderable for LeetcodeWeb {
    fn new(verbose: bool, config: LeetcodeConfig, cache: Arc<RwLock<Cache>>) -> Self {
        return LeetcodeWeb {
            verbose,
            config,
            cache,

            enable_cache: RefCell::new(false),
        };
    }

    fn render_config(&self) -> Vec<WebsiteContest> {
        let config = &self.config;
        return executor::block_on(self.render(&config.contests, &config.users));
    }

    fn website_name(&self) -> String {
        return String::from("leetcode");
    }

    fn render_live(&self) -> Vec<WebsiteContest> {
        *self.enable_cache.borrow_mut() = false;
        let contests = &self.config.live_contests;
        let users = &self.config.live_users;

        return executor::block_on(self.__render(contests, users));
    }
}
