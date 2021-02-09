use crate::{
    model::{config::LeetcodeConfig, website::WebsiteContest},
    service::cache::Cache,
};
use std::sync::{Arc, RwLock};


pub trait Renderable {
    fn render_config(&self) -> Vec<WebsiteContest>;
    fn website_name(&self) -> String;

    fn render_live(&self) -> Vec<WebsiteContest>;
    fn new(verbose: bool, config: LeetcodeConfig, cache: Arc<RwLock<Cache>>) -> Self
    where
        Self: Sized;
}