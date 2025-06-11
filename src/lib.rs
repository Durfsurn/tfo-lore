use std::{collections::BTreeMap, sync::Arc};

use chrono::NaiveDate;
use dominator::Dom;

use js_sys::Array;
use serde::Deserialize;
use wasm_bindgen::{
    JsCast, JsValue,
    prelude::{Closure, wasm_bindgen},
};

pub use macros::*;
mod macros;
pub use utilities::*;
use web_sys::{Element, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};
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
            .class("section-wrapper")
            .children(DATABASE.keys().enumerate().map(show_item))
            .after_inserted(|_| {
                do_scroll();
            })
            .into_dom()
    }
}

fn show_item((idx, date): (usize, &NaiveDate)) -> Dom {
    let htmls = DATABASE.get(date).unwrap().html.as_slice();

    section()
        .class([
            "section",
            "grid",
            "h-screen",
            "place-items-center",
            "transition-all",
            "duration-700",
            "ease-in-out",
        ])
        .apply(|s| {
            s.class(
                // Initial state: either active or hidden
                if idx == 0 {
                    ["opacity-100", "translate-y-0"]
                } else {
                    ["opacity-0", "translate-y-12"]
                },
            )
        })
        .child(
            div()
                .class(["flex", "flex-wrap", "justify-center", "gap-4"])
                .children(htmls.iter().map(|html| {
                    div()
                        .class(["discord-dark", "item"])
                        .after_inserted(move |el| el.set_inner_html(html))
                        .into_dom()
                }))
                .child(hr().into_dom())
                .into_dom(),
        )
        .into_dom()
}
fn do_scroll() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let sections = document.query_selector_all(".section").unwrap();

    let callback = Closure::wrap(Box::new(move |entries: Array, _: IntersectionObserver| {
        let mut max_ratio = 0.0;
        let mut most_visible: Option<Element> = None;

        // Find the entry with the largest intersectionRatio
        for entry_js in entries.iter() {
            let entry = entry_js.unchecked_into::<IntersectionObserverEntry>();
            if entry.is_intersecting() && entry.intersection_ratio() > max_ratio {
                max_ratio = entry.intersection_ratio();
                most_visible = Some(entry.target().unchecked_into::<Element>());
            }
        }

        // Add/remove Tailwind classes for visibility and animation
        for entry_js in entries.iter() {
            let entry = entry_js.unchecked_into::<IntersectionObserverEntry>();
            let target = entry.target().unchecked_into::<Element>();

            if let Some(ref visible_el) = most_visible {
                if target.is_same_node(Some(visible_el)) {
                    // Make this section visible
                    let _ = target.class_list().add_2("opacity-100", "translate-y-0");
                    let _ = target.class_list().remove_2("opacity-0", "translate-y-12");
                } else {
                    // Hide this section
                    let _ = target.class_list().remove_2("opacity-100", "translate-y-0");
                    let _ = target.class_list().add_2("opacity-0", "translate-y-12");
                }
            }
        }
    }) as Box<dyn FnMut(Array, IntersectionObserver)>);

    let observer_options = IntersectionObserverInit::new();
    observer_options.set_root_margin("0px");
    observer_options.set_threshold(&JsValue::from_f64(0.1));

    let observer = IntersectionObserver::new_with_options(
        callback.as_ref().unchecked_ref(),
        &observer_options,
    )
    .unwrap();

    callback.forget();

    for i in 0..sections.length() {
        let section = sections.get(i).unwrap().unchecked_into::<Element>();
        observer.observe(&section);
    }
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
