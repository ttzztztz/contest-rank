#[macro_use]
extern crate clap;
use crate::service::{cache, converter::convert_website_object, live::live, render};
use clap::App;
use service::handler;
use std::{path::Path, sync::Arc};

mod model;
mod service;
mod utils;
mod web;

fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    if matches.is_present("clear_cache") {
        runtime.block_on(cache::force_clear_cache());
        return;
    }

    let is_verbose = matches.is_present("verbose");
    if is_verbose {
        println!("[INFO] Currently in verbose mode");
    }

    let config_path = matches.value_of("config").unwrap_or("./conf.json");
    let config = service::config::read_config(config_path);

    if matches.is_present("show_config") {
        println!("🔧 Config loaded from json :");
        println!("{}", config.to_json());
        return;
    }

    if matches.is_present("show_config_path") {
        let path = Path::new(config_path);
        println!("{}", path.canonicalize().unwrap().display());
        return;
    }

    let settings = model::config::Settings {
        config,
        verbose: is_verbose,
    };
    let handlers = handler::handler_hashmap();

    for (website_name, handler) in handlers.iter() {
        if let Some(website_matches) = matches.subcommand_matches(website_name) {
            let mut settings = settings;

            if (handler.subcommand_match)(website_matches, &mut settings)
                && settings.config.write_to_file(config_path)
            {
                println!("[INFO] 🌟 Config written to path={}", config_path);
            } else {
                println!("[INFO] 😱 Config file unchanged, path={}", config_path);
            }

            return;
        }
    }

    let is_live = matches.is_present("live");
    if is_verbose && is_live {
        println!("[INFO] Currently in live mode");
    }
    let hide_submission = matches.is_present("hide_submission");
    if is_verbose && hide_submission {
        println!("[INFO] Submission info is hidden in output");
    }

    match handlers.get(&settings.config.website) {
        Some(handler) => {
            println!("[INFO] Prparing data, please wait...");

            if is_live {
                let website =
                    (handler.new)(is_verbose, settings.config.clone(), runtime.clone(), true);
                live(&settings.config.live, &website, hide_submission);
            } else {
                let website =
                    (handler.new)(is_verbose, settings.config.clone(), runtime.clone(), false);
                let website_contests = website.render();
                let render_object = convert_website_object(website_contests, is_live);
                render::render(render_object, hide_submission);
            }
        }
        None => {
            println!(
                "[INFO] No match handler for website={}",
                settings.config.website
            );
        }
    }
}
