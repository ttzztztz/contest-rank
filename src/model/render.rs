
#[derive(Clone)]
pub enum SubmissionStatus {
    Accepted,
    Unaccepted,
}

#[derive(Clone)]
pub struct Submission {
    pub fail_count: u32,
    pub finish_time: String,
    pub status: SubmissionStatus,
    pub score: u32,
}

pub struct User {
    pub username: String,
    pub country: String,
    pub finish_time: String,
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
}

pub struct Contest<T = User> {
    pub name: String,
    pub date: String,

    pub players: Vec<T>,
}

pub struct RenderObject {
    pub data: Vec<Contest>,
    pub aggregate: Vec<UserAggregate>,
}
