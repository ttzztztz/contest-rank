#[macro_use]
extern crate clap;
use clap::App;

mod model;
mod service;

fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let verbose_open = matches.occurrences_of("verbose") >= 1;
}
