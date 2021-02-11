use crate::model::{config::Config, config::Settings, website::WebsiteContest};
use clap::ArgMatches;
use std::sync::Arc;

pub trait Renderable {
    fn new(
        verbose: bool,
        config: Config,
        runtime: Arc<tokio::runtime::Runtime>,
        is_live: bool,
    ) -> Box<dyn Renderable>
    where
        Self: Sized;
    fn render(self: &Self) -> Vec<WebsiteContest>;
}

pub trait WebsiteTrait {
    fn website_name() -> &'static str;

    fn subcommand_match(website_matches: &ArgMatches, settings: &mut Settings) -> bool;
}
