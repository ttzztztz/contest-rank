use crate::{
    model::{
        config,
        renderable::{Renderable, WebsiteTrait},
    },
    web::leetcode::LeetcodeWeb,
};
use std::{collections::HashMap, sync::Arc};

pub struct HandlerHashMapValue {
    pub new: fn(bool, config::Config, Arc<tokio::runtime::Runtime>, bool) -> Box<dyn Renderable>,
    pub subcommand_match: fn(&clap::ArgMatches, &mut config::Settings) -> bool,
}

type HandlerHashMapType = HashMap<String, HandlerHashMapValue>;

macro_rules! add_website_to_hashmap {
    ($handler_hashmap: expr, $($name: tt),*) => {
        $($handler_hashmap.insert($name::website_name().to_string(), HandlerHashMapValue {
            new: $name::new,
            subcommand_match: $name::subcommand_match,
        });)*
    };
}

pub fn handler_hashmap() -> HandlerHashMapType {
    let mut handler_hashmap: HandlerHashMapType = HashMap::new();

    add_website_to_hashmap!(handler_hashmap, LeetcodeWeb);
    // more website can be added in the future
    handler_hashmap
}
