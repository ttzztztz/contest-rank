use crate::model::{config::Settings, website::WebsiteContest, renderable::Renderable};

pub struct StretchWeb {}

impl Renderable for StretchWeb {
    fn render(_: Settings) -> Vec<WebsiteContest> {
        let render_object = Vec::<WebsiteContest>::new();

        return render_object;
    }

    fn website_name() -> String {
        return String::from("stretch");
    }
}
