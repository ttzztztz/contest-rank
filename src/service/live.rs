use super::{converter::convert_website_object, render};
use crate::model::{config::LiveConfig, renderable::Renderable};
use chrono::Local;
use futures::executor;

pub fn live(config: &LiveConfig, website: &Box<dyn Renderable>) {
    let start_time = Local::now().timestamp();
    while Local::now().timestamp() - start_time < config.last {
        let content = website.render_live();
        let render_object = convert_website_object(content, true);
        render::render(render_object);

        executor::block_on(tokio::time::sleep(std::time::Duration::from_secs(
            config.interval,
        )));
    }
}
