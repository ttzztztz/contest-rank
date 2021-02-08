#[macro_use]
extern crate clap;
use clap::App;
use model::{config::Settings, renderable::Renderable};
use service::{cache::clear_cache, converter::convert_website_object, render};
use std::collections::HashMap;
use web::leetcode::LeetcodeWeb;

mod model;
mod service;
mod utils;
mod web;

fn handler_hashmap(settings: &Settings) -> HashMap<String, Box<dyn Renderable>> {
    let mut handler_hashmap: HashMap<String, Box<dyn Renderable>> = HashMap::new();
    let verbose = settings.verbose;

    let leetcode_web = Box::from(LeetcodeWeb {
        verbose,
        config: settings.config.leetcode.clone(),
    });
    handler_hashmap.insert(leetcode_web.website_name(), leetcode_web);

    // We can support more website in the future!
    handler_hashmap
}

fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let force_clear_cache = matches.occurrences_of("clear_cache") >= 1;
    if force_clear_cache {
        clear_cache().unwrap();
        println!("🌟 [Cache] Cache cleared!");
    }

    let verbose = matches.occurrences_of("verbose") >= 1;
    let config_path = matches.value_of("config").unwrap_or("./conf.json");
    let config = service::config::read_config(config_path);

    let settings = model::config::Settings { config, verbose };
    let handlers = handler_hashmap(&settings);

    match handlers.get(&settings.config.website) {
        Some(website) => {
            let website_contests = website.render_config();
            let render_object = convert_website_object(website_contests);
            render::render(render_object);
        }
        None => {
            println!("No match handler for website {}", settings.config.website);
        }
    }
}
