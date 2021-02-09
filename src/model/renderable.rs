use crate::{
    model::{config::LeetcodeConfig, website::WebsiteContest},
    service::cache::Cache,
};
use std::sync::{Arc, RwLock};

pub trait Renderable {
    fn new(verbose: bool, config: LeetcodeConfig, cache: Arc<RwLock<Cache>>) -> Self
    where
        Self: Sized;
    fn render_config(&self, runtime: &tokio::runtime::Runtime) -> Vec<WebsiteContest>;
    fn website_name(&self) -> String;

    fn render_live(&self) -> Vec<WebsiteContest>;
}
