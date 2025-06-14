use std::{collections::BTreeMap, sync::Arc};

use chrono::NaiveDate;
use dominator::{Dom, events};
use futures_signals::signal::Mutable;
use gloo::utils::window;
use serde::Deserialize;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};

pub use macros::*;
mod macros;
pub use utilities::*;
use web_sys::{Element, Event, HtmlElement, HtmlVideoElement, Node};
mod utilities;

pub fn is_local() -> bool {
    if let Ok(hostname) = window().location().hostname() {
        // Check IPv6 loopback and localhost
        return  hostname == "[::1]" || hostname == "::1" || hostname == "localhost" || hostname == "127.0.0.1";
    }

    false
}
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

struct Page {
    pride: Mutable<Arc<str>>,
}
static PAGE: once_cell::sync::Lazy<Arc<Page>> = once_cell::sync::Lazy::new(|| {
    Arc::new(Page {
        pride: Mutable::new("rainbow-pride".into()),
    })
});

impl Page {
    fn render() -> Dom {
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
            .apply_if(is_local(), |p| {
                p.children(DATABASE.keys().rev().enumerate().map(show_item))
            })
            .apply_if(!is_local(), |p| {
                p.children(DATABASE.keys().enumerate().map(show_item))
            })
            .child(
                div()
                    .class("border-buttons")
                    .children([
                        button()
                            .class("rainbow-pride")
                            .class_signal(
                                "selected",
                                PAGE.pride.signal_ref(|c| c.as_ref() == "rainbow-pride"),
                            )
                            .text("Rainbow")
                            .event({
                                move |_: events::Click| {
                                    PAGE.pride.set("rainbow-pride".into());
                                }
                            })
                            .into_dom(),
                        button()
                            .class("trans-pride")
                            .class_signal(
                                "selected",
                                PAGE.pride.signal_ref(|c| c.as_ref() == "trans-pride"),
                            )
                            .text("Trans")
                            .event({
                                move |_: events::Click| {
                                    PAGE.pride.set("trans-pride".into());
                                }
                            })
                            .into_dom(),
                    ])
                    .into_dom(),
            )
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
                    "gap-8",
                ])
                .children(htmls.iter().map(|html| {
                    div()
                        .class(["discord-dark", "item", "self-center"])
                        .class_signal(
                            "rainbow-pride",
                            PAGE.pride.signal_ref(|c| c.as_ref() == "rainbow-pride"),
                        )
                        .class_signal(
                            "trans-pride",
                            PAGE.pride.signal_ref(|c| c.as_ref() == "trans-pride"),
                        )
                        .after_inserted(move |el| {
                            el.set_inner_html(html);
                            insert_br_before_span_with_img_class();
                            setup_emoji_click_handler();
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

    dominator::append_dom(&dominator::body(), Page::render());

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
    let selector = "span > img.emoji:not(.jumboable)";
    let node_list = document.query_selector_all(selector).unwrap();

    for i in 0..node_list.length() {
        if let Some(img_el) = node_list.item(i) {
            // Skip if the img has class 'jumboable'
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

pub fn setup_emoji_click_handler() {
    let document = window().document().unwrap();

    let selector = "div.reaction__23977";
    let node_list = document.query_selector_all(selector).unwrap();

    for i in 0..node_list.length() {
        let div = node_list.item(i).unwrap();
        let div_el = div.dyn_into::<Element>().unwrap();

        // Skip if already initialized
        if div_el.has_attribute("data-emoji-initialized") {
            continue;
        }
        let _ = div_el.set_attribute("data-emoji-initialized", "true");

        let closure = {
            let div_el = div_el.clone();

            Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                let class_list = div_el.class_list();

                let is_removing = class_list.contains("reactionMe__23977");

                // Toggle the class
                let _ = if is_removing {
                    class_list.remove_1("reactionMe__23977")
                } else {
                    class_list.add_1("reactionMe__23977")
                };

                // Access div's first child and update count
                if let Some(first_child_el) = div_el.first_element_child() {
                    if let Some(last_child) = first_child_el.last_element_child() {
                        let text = last_child.text_content().unwrap_or_default();
                        let trimmed = text.trim();

                        if let Ok(num) = trimmed.parse::<i32>() {
                            let new_val = if is_removing {
                                format!("{}", num.saturating_sub(1))
                            } else {
                                format!("{}", num + 1)
                            };
                            last_child.set_text_content(Some(&new_val));
                        }
                    }
                }
            })
        };

        let _ = div_el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget(); // Prevent dropping
    }
}
