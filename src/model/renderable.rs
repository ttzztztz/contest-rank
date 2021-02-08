use crate::model::website::WebsiteContest;

pub trait Renderable {
    fn render_config(&self) -> Vec<WebsiteContest>;
    fn render(&self, contests: &Vec<String>, users: &Vec<String>) -> Vec<WebsiteContest>;
    fn website_name(&self) -> String;
}
