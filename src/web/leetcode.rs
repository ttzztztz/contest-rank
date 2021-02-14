use crate::{
    model::{
        config::{Config, Settings, WebsiteConfig},
        render::{Submission, SubmissionStatus},
        renderable::{Renderable, WebsiteTrait},
        website::{WebsiteContest, WebsiteUser},
    },
    service::cache,
    utils::{null, request},
};
use clap::ArgMatches;
use futures::future;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
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
    pub config: WebsiteConfig,

    pub enable_cache: bool,
    pub is_live: bool,
    pub runtime: Arc<tokio::runtime::Runtime>,
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

        let cache_key = format!("lc_{}{}_{}", contest_type, contest_id, page);
        let url = format!(
            "https://leetcode.com/contest/api/ranking/{contest_type}-contest-{id}?pagination={page}&region=global",
            id = contest_id,
            contest_type = contest_type_full,
            page = page
        );

        if self.enable_cache {
            if let Some(memo) = cache::get_cache::<LeetcodeRankRequest>(&cache_key).await {
                if self.verbose {
                    println!("[INFO] Cache hit request url={}", url);
                }
                return Ok(memo);
            }
        }

        let res = request::send_request::<LeetcodeRankRequest>(&url).await?;
        if self.enable_cache && res.is_past {
            cache::set_cache(&cache_key, &res).await;
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
        while !searching_players.is_empty() && (page - 1u32) * 25u32 < self.config.max_rank {
            let mut ranks = vec![];
            for page_offset in 0u32..self.config.concurrent {
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

            let mut zero_player_page_cnt = 0;
            for rank_result in ranks.iter() {
                match rank_result {
                    Ok(rank) => {
                        assert_eq!(rank.submissions.len(), rank.total_rank.len());

                        let playeres_in_page = rank.submissions.len();
                        if playeres_in_page == 0 {
                            zero_player_page_cnt += 1;
                        }
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
                                        let submission_status;
                                        if self.is_live {
                                            submission_status = SubmissionStatus::Pending;
                                        } else {
                                            submission_status = SubmissionStatus::Unaccepted;
                                        }

                                        submissions_vec.push(Submission {
                                            fail_count: 0,
                                            finish_time: 0,
                                            status: submission_status,
                                            score: 0,
                                            title: format!("T{}", question_index + 1),
                                        });
                                    }
                                    Some(submission) => {
                                        submissions_vec.push(Submission {
                                            fail_count: submission.fail_count,
                                            finish_time: submission.date - contest_info.start_time,
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
                    Err(err) => {
                        println!("[Error] When fetching rank result, e={}", err);
                    }
                }
            }

            if self.verbose && zero_player_page_cnt == self.config.concurrent {
                println!("[INFO] Exit searching, ignoring max_page, already hit the end of page");
            }
            page += self.config.concurrent;
        }

        return Ok(WebsiteContest {
            name: String::from("LeetCode ") + &contest_info.title,
            date: contest_info.start_time,
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
                Err(err) => {
                    println!("[ERROR] When fetching contest {}", err);
                }
            }
        }
        return web_contests;
    }

    fn render_live(&self) -> Vec<WebsiteContest> {
        let contests = &self.config.live_contests;
        let users = &self.config.live_users;

        return self.runtime.block_on(self.__render(contests, users));
    }

    fn render_contest(&self) -> Vec<WebsiteContest> {
        let config = &self.config;
        return self
            .runtime
            .block_on(self.__render(&config.contests, &config.users));
    }
}

impl Renderable for LeetcodeWeb {
    fn new(
        verbose: bool,
        config: Config,
        runtime: Arc<tokio::runtime::Runtime>,
        is_live: bool,
    ) -> Box<dyn Renderable> {
        let mut instance = LeetcodeWeb {
            verbose,
            config: config.leetcode,
            runtime,

            enable_cache: false,
            is_live,
        };

        if is_live {
            instance.enable_cache = false;
        } else {
            instance.enable_cache = instance.config.cache;
        }

        return Box::new(instance);
    }

    fn render(&self) -> Vec<WebsiteContest> {
        if self.is_live {
            return self.render_live();
        } else {
            return self.render_contest();
        }
    }
}

impl WebsiteTrait for LeetcodeWeb {
    fn website_name() -> &'static str {
        return "leetcode";
    }

    fn subcommand_match(website_matches: &ArgMatches, settings: &mut Settings) -> bool {
        match website_matches.subcommand() {
            ("user", Some(arg_matches)) => match arg_matches.subcommand() {
                ("add", Some(arg_matches)) => {
                    let username = arg_matches.value_of("username").unwrap();

                    let vec;
                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_users;
                    } else {
                        vec = &mut settings.config.leetcode.users;
                    }

                    vec.push(username.to_string());
                    println!("[INFO] 🔧 Added user {} to LeetCode config", username);
                    return true;
                }
                ("truncate", Some(arg_matches)) => {
                    let vec;
                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_users;
                    } else {
                        vec = &mut settings.config.leetcode.users;
                    }

                    vec.clear();
                    println!("[INFO] 🔧 Cleared all users in LeetCode config");
                    return true;
                }
                ("delete", Some(arg_matches)) => {
                    let username = arg_matches.value_of("username").unwrap();

                    let vec;
                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_users;
                    } else {
                        vec = &mut settings.config.leetcode.users;
                    }

                    match vec.iter().position(move |val| val == username) {
                        Some(idx) => {
                            vec.remove(idx);
                            println!("[INFO] 🔧 Remove user {} to LeetCode config", username);
                            return true;
                        }
                        None => {
                            println!("[INFO] ❌ Username {} doesn't exist", username);
                            return false;
                        }
                    }
                }
                _ => {}
            },
            ("contest", Some(arg_matches)) => match arg_matches.subcommand() {
                ("add", Some(arg_matches)) => {
                    let contest_id = arg_matches.value_of("contest_id").unwrap();

                    let vec;
                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_contests;
                    } else {
                        vec = &mut settings.config.leetcode.contests;
                    }

                    vec.push(contest_id.to_string());
                    println!(
                        "[INFO] 🔧 Added contest_id {} to LeetCode config",
                        contest_id
                    );
                    return true;
                }
                ("truncate", Some(arg_matches)) => {
                    let vec;

                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_contests;
                    } else {
                        vec = &mut settings.config.leetcode.contests;
                    }

                    vec.clear();
                    println!("[INFO] 🔧 Cleared all contest_ids in LeetCode config");
                    return true;
                }
                ("delete", Some(arg_matches)) => {
                    let contest_id = arg_matches.value_of("contest_id").unwrap();

                    let vec;
                    if arg_matches.is_present("live") {
                        vec = &mut settings.config.leetcode.live_contests;
                    } else {
                        vec = &mut settings.config.leetcode.contests;
                    }

                    match vec.iter().position(move |val| val == contest_id) {
                        Some(idx) => {
                            vec.remove(idx);
                            println!(
                                "[INFO] 🔧 Added contest_id {} to LeetCode config",
                                contest_id
                            );
                            return true;
                        }
                        None => {
                            println!("[INFO] ❌ Contest {} doesn't exist", contest_id);
                            return false;
                        }
                    }
                }
                _ => {}
            },
            ("set", _) => {
                settings.config.website = String::from("leetcode");
                println!("[INFO] 🔧 Set website to LeetCode",);
                return true;
            }
            _ => {}
        }

        return false;
    }
}
