use crate::{
    model::{config::LiveConfig, renderable::Renderable},
    service::{converter::convert_website_object, render},
};
use chrono::Local;
use std::thread;

pub fn live(config: &LiveConfig, website: &Box<dyn Renderable>, hide_submission: bool) {
    let start_time = Local::now().timestamp();
    while Local::now().timestamp() - start_time < config.last {
        let content = website.render();
        let render_object = convert_website_object(content, true);
        render::render(render_object, hide_submission);

        thread::sleep(std::time::Duration::from_secs(config.interval));
    }

    println!("[INFO] Live ended");
}
