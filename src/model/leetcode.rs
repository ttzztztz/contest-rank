use serde::Serialize;

#[derive(Serialize)]
pub struct SubmissionItem<'a> {
    pub data_region: &'a str,
    pub fail_count: u32,
    pub date: u64,
    pub question_id: u32,
    pub submission_id: u32,
}

#[derive(Serialize)]
pub struct RankItem<'a> {
    pub country_name: &'a str,
    pub finish_time: u64,
    pub rank: u32,
    pub score: u16,
    pub username: &'a str,
    pub data_region: &'a str,
}
#[derive(Serialize)]
pub struct LeetcodeRequest<'a> {
    pub time: f64,
    pub submissions: Vec<SubmissionItem<'a>>,
    pub user_num: u64,
    pub is_past: bool,
    pub total_rank: Vec<RankItem<'a>>,
}
