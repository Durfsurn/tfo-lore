use std::{collections::BTreeMap, sync::Arc};

use chrono::NaiveDate;
use dominator::Dom;

use serde::Deserialize;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

pub use macros::*;
mod macros;
pub use utilities::*;
mod utilities;

#[derive(Debug, Deserialize)]
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
    fn render(page: &Arc<Self>) -> Dom {
        div()
            .class(["grid", "h-screen", "place-items-center"])
            .children(DATABASE.keys().enumerate().map(show_item))
            .into_dom()
    }
}

fn show_item((idx, date): (usize, &NaiveDate)) -> Dom {
    let htmls = DATABASE.get(date).unwrap().html.as_slice();
    div().class(["flex", "flex-wrap", "justify-center", "gap-4"])
        .children(htmls.iter().map(|html| {
            div()
                .class(["discord-dark", "item"])
                .after_inserted(move |el| el.set_inner_html(html))
                .into_dom()
        }))
        .child(hr().into_dom())
        .into_dom()
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    {
        let el = gloo::utils::document()
            .get_element_by_id("logo-fade")
            .unwrap();
        add_element_class(el, "fade-out");
        gloo::timers::callback::Timeout::new(1000, || {
            gloo::utils::document()
                .get_element_by_id("logo-fade")
                .map(|el| el.remove())
                .unwrap();
        })
        .forget();
    }

    let page = Page::new();
    dominator::append_dom(&dominator::body(), Page::render(&page));

    Ok(())
}
