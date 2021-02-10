use crate::model::{config::LeetcodeConfig, website::WebsiteContest};
use std::sync::Arc;

pub trait Renderable {
    fn new(verbose: bool, config: LeetcodeConfig, runtime: Arc<tokio::runtime::Runtime>) -> Self
    where
        Self: Sized;
    fn render_config(&self) -> Vec<WebsiteContest>;
    fn website_name(&self) -> String;

    fn render_live(&self) -> Vec<WebsiteContest>;
}
