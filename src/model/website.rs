use super::render::{Contest, Submission};

#[derive(Clone)]
pub struct WebsiteUser {
    pub username: String,
    pub country: String,
    pub finish_time: i64,
    pub global_rank: u32,
    pub score: u32,

    pub submissions: Vec<Submission>,
}

pub type WebsiteContest = Contest<WebsiteUser>;
