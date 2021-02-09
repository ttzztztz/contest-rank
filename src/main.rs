#[macro_use]
extern crate clap;
use clap::App;
use model::{config::Settings, renderable::Renderable};
use service::{
    cache::{clear_cache, Cache},
    converter::convert_website_object,
    live::live,
    render,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use web::leetcode::LeetcodeWeb;

mod model;
mod service;
mod utils;
mod web;

fn handler_hashmap(
    settings: &Settings,
    cache: Arc<RwLock<Cache>>,
) -> HashMap<String, Box<dyn Renderable>> {
    let mut handler_hashmap: HashMap<String, Box<dyn Renderable>> = HashMap::new();
    let verbose = settings.verbose;

    let leetcode_web = Box::from(LeetcodeWeb::new(
        verbose,
        settings.config.leetcode.clone(),
        cache,
    ));
    handler_hashmap.insert(leetcode_web.website_name(), leetcode_web);

    // We can support more website in the future!
    handler_hashmap
}

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    let cache = Arc::new(RwLock::new(Cache::new()));

    let force_clear_cache = matches.occurrences_of("clear_cache") >= 1;
    if force_clear_cache {
        clear_cache().unwrap();
        println!("🌟 [Cache] Cache cleared!");
    }

    let is_verbose = matches.occurrences_of("verbose") >= 1;
    if is_verbose {
        println!("[INFO] currently in verbose mode");
    }

    let is_live = matches.occurrences_of("live") >= 1;
    if is_verbose && is_live {
        println!("[INFO] currently in live mode");
    }

    let config_path = matches.value_of("config").unwrap_or("./conf.json");
    let config = service::config::read_config(config_path);

    let settings = model::config::Settings {
        config,
        verbose: is_verbose,
    };
    let handlers = handler_hashmap(&settings, cache);

    match handlers.get(&settings.config.website) {
        Some(website) => {
            println!("[INFO] We are prparing data, please wait...");

            let website_contests;
            if is_live {
                live(&settings.config.live, website).await;
            } else {
                website_contests = website.render_config();
                let render_object = convert_website_object(website_contests, is_live);
                render::render(render_object);
            }
        }
        None => {
            println!("No match handler for website {}", settings.config.website);
        }
    }
}
