#[macro_use]
extern crate clap;
use clap::App;
use model::renderable::{RenderFunction, Renderable};
use service::{converter::convert_website_object, render};
use std::collections::HashMap;
use web::{leetcode::LeetcodeWeb, stretch::StretchWeb};

mod model;
mod service;
mod web;
mod utils;

fn handler_hashmap() -> HashMap<String, RenderFunction> {
    let mut handler_hashmap: HashMap<String, RenderFunction> = HashMap::new();
    handler_hashmap.insert(LeetcodeWeb::website_name(), LeetcodeWeb::render);
    handler_hashmap.insert(StretchWeb::website_name(), StretchWeb::render);
    handler_hashmap
}

fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let verbose = matches.occurrences_of("verbose") >= 1;
    let config_path = matches.value_of("config").unwrap_or("./conf.json");
    let config = service::config::read_config(config_path);

    let settings = model::config::Settings { config, verbose };
    let handlers = handler_hashmap();

    let website = settings.config.website.clone();
    match handlers.get(&website) {
        Some(handler) => {
            let website_contests = handler(settings);
            let render_object = convert_website_object(website_contests);
            render::render(render_object);
        }
        None => {
            println!("No match handler for website {}", settings.config.website);
        }
    }
}
