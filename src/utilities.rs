#![allow(clippy::too_many_arguments)]
use {itertools::Itertools, web_sys::Element};

mutablex::mutable_x!(2);
mutablex::mutable_x!(3);
mutablex::mutable_x!(4);
mutablex::mutable_x!(5);
mutablex::mutable_x!(6);
mutablex::mutable_x!(7);
mutablex::mutable_x!(8);
mutablex::mutable_x!(9);

pub const NBSP: &str = "\u{00A0}";

pub fn dash_str() -> &'static str {
    "-"
}

pub fn add_element_class(el: impl Into<Element>, class: impl AsRef<str>) {
    let class = class.as_ref();
    let el: Element = el.into();
    let existing_class = el.get_attribute("class").unwrap_or_default();

    if !existing_class.contains(class) {
        let _ = el.set_attribute("class", format!("{} {}", existing_class, class).trim());
    }
}
pub fn remove_element_class(el: impl Into<Element>, class: impl AsRef<str>) {
    let class = class.as_ref();
    let el: Element = el.into();
    let existing_class = el.get_attribute("class").unwrap_or_default();

    let _ = el.set_attribute("class", existing_class.replace(class, "").trim());
}

pub fn remove_element_styles(el: impl Into<Element>, styles: Vec<String>) {
    let el: Element = el.into();
    let existing_style = el.get_attribute("style").unwrap_or_default();

    let existing_styles = (if existing_style.is_empty() {
        Vec::new()
    } else {
        existing_style.split(';').collect_vec()
    })
    .into_iter()
    .map(|es| {
        let mut style = es.split(": ");
        (style.next().unwrap_or_default().to_string(), style.next().unwrap_or_default().to_string())
    })
    .collect_vec();

    let mut new_styles = existing_styles;
    for new in styles {
        if let Some(idx) = new_styles.iter().position(|(a, _)| a.trim() == new.trim()) {
            new_styles.remove(idx);
        }
    }
    el.set_attribute(
        "style",
        &new_styles
            .into_iter()
            .filter(|(a, b)| !a.trim().is_empty() && !b.trim().is_empty())
            .map(|(a, b)| format!("{}: {}", a.trim(), b.trim()))
            .join("; "),
    )
    .unwrap();
}
pub fn update_element_style(el: impl Into<Element>, styles: Vec<(String, String)>) {
    let el: Element = el.into();
    let existing_style = el.get_attribute("style").unwrap_or_default();

    let existing_styles = (if existing_style.is_empty() {
        Vec::new()
    } else {
        existing_style.split(';').collect_vec()
    })
    .into_iter()
    .map(|es| {
        let mut style = es.split(": ");
        (style.next().unwrap_or_default().to_string(), style.next().unwrap_or_default().to_string())
    })
    .collect_vec();

    let mut new_styles = existing_styles;
    for new in styles {
        if let Some(idx) = new_styles.iter().position(|(a, _)| a.trim() == new.0.trim()) {
            new_styles.remove(idx);
            new_styles.push(new);
        } else {
            new_styles.push(new);
        }
    }
    el.set_attribute(
        "style",
        &new_styles
            .into_iter()
            .filter(|(a, b)| !a.trim().is_empty() && !b.trim().is_empty())
            .map(|(a, b)| format!("{}: {}", a.trim(), b.trim()))
            .join("; "),
    )
    .unwrap();
}