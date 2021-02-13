
#[derive(Clone, PartialEq)]
pub enum SubmissionStatus {
    Accepted,
    Unaccepted,
    Pending
}

#[derive(Clone)]
pub struct Submission {
    pub fail_count: u32,
    pub finish_time: i64,
    pub status: SubmissionStatus,
    pub score: u32,
    pub title: String,
}

pub struct User {
    pub username: String,
    pub country: String,
    pub finish_time: i64,
    pub global_rank: u32,
    pub score: u32,
    pub local_rank: u32,

    pub submissions: Vec<Submission>,
}

pub struct UserAggregate {
    pub username: String,
    pub country: String,

    pub win_count: u32,
    pub attend_count: u32,
    pub total_score: u32,
    pub total_time: i64,
}

#[derive(Clone)]
pub struct Contest<T = User> {
    pub name: String,
    pub date: i64,

    pub players: Vec<T>,
}

pub struct RenderObject {
    pub data: Vec<Contest>,
    pub aggregate: Vec<UserAggregate>,

    pub is_live: bool,
}
