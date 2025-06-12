use std::{collections::BTreeMap, sync::Arc};

use chrono::NaiveDate;
use dominator::Dom;
use gloo::utils::window;
use serde::Deserialize;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};

pub use macros::*;
mod macros;
pub use utilities::*;
use web_sys::{Element, HtmlElement, HtmlVideoElement, Node};
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
            .children(DATABASE.keys().rev().enumerate().map(show_item))
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
                        .after_inserted(move |el| {
                            el.set_inner_html(html);
                            insert_br_before_span_with_img_class()
                        })
                        .into_dom()
                }))
                .into_dom(),
        )
        .into_dom()
}

#[wasm_bindgen(start)]
fn main_js() -> Result<(), wasm_bindgen::JsValue> {
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

    handle_video();

    Ok(())
}

fn handle_video() {
    let document = window().document().unwrap();

    // Select both video elements and .cover__6eb54 elements
    let targets = document.query_selector_all("video, .cover__6eb54").unwrap();

    for i in 0..targets.length() {
        let node = targets.item(i).unwrap();

        let closure = Closure::wrap(Box::new({
            let node = node.clone();
            move || {
                let maybe_video = if let Ok(video) = node.clone().dyn_into::<HtmlVideoElement>() {
                    Some(video)
                } else if let Ok(cover) = node.clone().dyn_into::<HtmlElement>() {
                    cover.parent_element().and_then(|parent| {
                        let children = parent.children();
                        for j in 0..children.length() {
                            let child = children.item(j).unwrap();
                            if let Ok(video) = child.dyn_into::<HtmlVideoElement>() {
                                return Some(video);
                            }
                        }
                        None
                    })
                } else {
                    None
                };

                if let Some(video) = maybe_video {
                    toggle_video_and_play(&video);
                }
            }
        }) as Box<dyn FnMut()>);

        node.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget(); // prevent GC

        // If it's a video, also add an 'ended' listener
        if let Ok(video) = node.dyn_into::<HtmlVideoElement>() {
            let ended_closure = Closure::wrap(Box::new({
                let video = video.clone();
                move || {
                    show_play_controls(&video);
                }
            }) as Box<dyn FnMut()>);

            video
                .add_event_listener_with_callback("ended", ended_closure.as_ref().unchecked_ref())
                .unwrap();
            ended_closure.forget(); // prevent GC
        }
    }
}

fn toggle_video_and_play(video: &HtmlVideoElement) {
    let is_paused = video.paused();

    if let Some(parent) = video.parent_element() {
        let siblings = parent.children();
        for i in 0..siblings.length() {
            let sibling = siblings.item(i).unwrap();
            if sibling.get_attribute("aria-label").as_deref() == Some("Play") {
                if is_paused {
                    let _ = video.play();
                    sibling.set_attribute("hidden", "true").ok();
                } else {
                    video.pause().ok();
                    sibling.remove_attribute("hidden").ok();
                }
            }
        }
    }
}

// Show the "Play" button and the .cover__6eb54 again when video ends
fn show_play_controls(video: &HtmlVideoElement) {
    if let Some(parent) = video.parent_element() {
        let siblings = parent.children();
        for i in 0..siblings.length() {
            let sibling = siblings.item(i).unwrap();

            if sibling.get_attribute("aria-label").as_deref() == Some("Play") {
                sibling.remove_attribute("hidden").ok();
            }

            if sibling.class_list().contains("cover__6eb54") {
                sibling.remove_attribute("hidden").ok();
            }
        }
    }
}

fn insert_br_before_span_with_img_class() {
    let document = window().document().unwrap();
    let selector = "span > img.emoji";
    let node_list = document.query_selector_all(selector).unwrap();

    for i in 0..node_list.length() {
        if let Some(img_el) = node_list.item(i) {
            if let Some(span_el) = img_el.parent_element() {
                if let Some(parent_node) = span_el.parent_node() {
                    // Check if there's already a <br> right before the span
                    if let Some(prev_sibling) = span_el.previous_sibling() {
                        if prev_sibling.node_type() == Node::ELEMENT_NODE {
                            if let Some(prev_elem) = prev_sibling.dyn_ref::<Element>() {
                                if prev_elem.tag_name().to_lowercase() == "br" {
                                    continue; // Already has a <br> before, skip
                                }
                            }
                        }
                    }

                    // No <br> before, insert one
                    let br = document.create_element("br").unwrap();
                    parent_node
                        .insert_before(&br, Some(&span_el))
                        .expect("Failed to insert <br>");
                }
            }
        }
    }
}
