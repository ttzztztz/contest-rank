use crate::model::{config::Settings, website::WebsiteContest};

pub type RenderFunction = fn(settings: Settings) -> Vec<WebsiteContest>;

pub trait Renderable {
    fn render(settings: Settings) -> Vec<WebsiteContest>;
    fn website_name() -> String;
}
