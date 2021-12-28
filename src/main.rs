use handlebars::Handlebars;
use serde::{Serialize, Deserialize};
use warp::Filter;
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug)]
pub struct Content {
    pub categories: Vec<Category>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Category {
    pub title: String,
    pub entries: Vec<Entry>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub favicon: String
}


struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: std::sync::Arc<T>,
}

fn render<T>(template: WithTemplate<T>, hbs: std::sync::Arc<Handlebars>) -> impl warp::Reply
    where T: Serialize + Debug
{
    let render = hbs
        .render(template.name, &*template.value)
        .unwrap_or_else(|err| err.to_string());

    warp::reply::html(render)
}

#[tokio::main]
async fn main() {
    let lp_content_str = std::fs::read_to_string("./content.json").unwrap();
    let lp_content_struct: Content = serde_json::from_str(lp_content_str.as_str()).unwrap();
    let lp_content = std::sync::Arc::new(lp_content_struct);

    let mut hb = Handlebars::new();
    hb.register_template_file("launchpad", "./launchpad.template.html")
        .unwrap();

    let hb = std::sync::Arc::new(hb);

    let handlebars = move || render(WithTemplate {
        name: "launchpad",
        value: lp_content.clone(),
    }, hb.clone());

    let lp = warp::get()
        .and(warp::path::end())
        .map(handlebars);

    let lp_css = warp::get()
        .and(warp::path!("style.css"))
        .and(warp::fs::file("./launchpad.style.css"));

    let lp_fallback_favicon = warp::get()
        .and(warp::path!("fallback.png"))
        .and(warp::fs::file("./fallback_favicon.png"));

    let launchpad = warp::any()
        .and(
            lp
                .or(lp_css)
                .or(lp_fallback_favicon)
        );

    println!("Listening on http://0.0.0.0:1111");
    warp::serve(launchpad).run(([0, 0, 0, 0], 1111)).await;
}
