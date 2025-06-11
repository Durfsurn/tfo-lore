use std::{collections::BTreeMap, sync::Arc};

use chrono::NaiveDate;
use dominator::Dom;
use serde::Deserialize;
use wasm_bindgen::prelude::wasm_bindgen;

pub use macros::*;
mod macros;
pub use utilities::*;
mod utilities;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Item {
    html: Vec<String>,
    link: String,
}

static DATABASE: once_cell::sync::Lazy<Arc<BTreeMap<NaiveDate, Item>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(serde_json::from_slice(include_bytes!("../events.json").as_slice()).unwrap())
    });

struct Page {}
impl Page {
    fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    fn render(_page: &Arc<Self>) -> Dom {
        div()
            .class([
                "w-screen",
                "h-screen",
                "overflow-y-scroll",
                "scroll-smooth",
                "snap-y",
                "snap-mandatory",
            ])
            .child(
                div()
                    .class([
                        "fixed",
                        "top-0",
                        "left-0",
                        "w-full",
                        "h-full",
                        "z-0",
                        "pointer-events-none",
                        "bg-no-repeat",
                        "bg-center",
                        "bg-cover",
                    ])
                    .class(["curtains-background"])
                    .into_dom(),
            )
            .children(DATABASE.keys().enumerate().map(show_item))
            .into_dom()
    }
}
fn show_item((_, date): (usize, &NaiveDate)) -> Dom {
    let htmls = DATABASE.get(date).unwrap().html.as_slice();

    section()
        .class([
            "section",
            "snap-start", // Snap anchor point
            "w-full",
            "h-screen", // Full viewport height
            "flex",
            "items-center",
            "justify-center",
            "relative",
        ])
        .child(
            div()
                .class([
                    "flex",
                    "flex-wrap",
                    "justify-center",
                    "items-stretch",
                    "gap-4",
                ])
                .children(htmls.iter().map(|html| {
                    div()
                        .class(["discord-dark", "item", "self-center"])
                        .after_inserted(move |el| el.set_inner_html(html))
                        .into_dom()
                }))
                .into_dom(),
        )
        .into_dom()
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), wasm_bindgen::JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    if let Some(el) = gloo::utils::document().get_element_by_id("logo-fade") {
        add_element_class(el.clone(), "fade-out");
        gloo::timers::callback::Timeout::new(1000, move || {
            el.remove();
        })
        .forget();
    }

    let page = Page::new();
    dominator::append_dom(&dominator::body(), Page::render(&page));

    Ok(())
}
