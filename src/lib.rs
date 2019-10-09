mod utils;

#[macro_use]
extern crate lazy_static;
extern crate serde_derive;

use std::sync::Mutex;
use std::collections::HashMap;

use serde_json::json;
use wasm_bindgen::prelude::*;
use pulldown_cmark::{Parser, Options, html, Event, Tag, CowStr};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref OPTIONS: Mutex<HashMap<String, serde_json::Value>> = {
        let mut m = HashMap::new();
        m.insert("headerLevelStart".to_string(), json!(false));
        m.insert("literalMidWordAsterisks".to_string(), json!(false));
        m.insert("noHeaderId".to_string(), json!(false));
        m.insert("simpleLineBreaks".to_string(), json!(false));
        m.insert("strikethrough".to_string(), json!(false));
        m.insert("tables".to_string(), json!(false));
        m.insert("tasklists".to_string(), json!(false));
        Mutex::new(m)
    };
}

#[wasm_bindgen]
pub fn getOption(key: &str) -> JsValue {
    JsValue::from_serde(&OPTIONS.lock().unwrap()[key]).unwrap()
}

#[wasm_bindgen]
pub fn getOptions() -> JsValue {
    JsValue::from_serde(&OPTIONS.lock().unwrap().to_owned()).unwrap()
}

#[wasm_bindgen]
pub fn setOption(key: String, value: JsValue) {
    OPTIONS.lock().unwrap().insert(key, value.into_serde().unwrap());
}

#[wasm_bindgen]
pub struct Converter {
    options: HashMap<String, serde_json::Value>,
}

#[wasm_bindgen]
impl Converter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Converter {
        Converter { options: OPTIONS.lock().unwrap().clone() }
    }

    pub fn setOption(&mut self, key: String, value: JsValue)
    {
        self.options.insert(key, value.into_serde().unwrap());
    }

    pub fn getOption(&self, key: &str) -> JsValue {
        JsValue::from_serde(&self.options[key]).unwrap()
    }

    pub fn getOptions(&self) -> JsValue {
        JsValue::from_serde(&self.options).unwrap()
    }

    pub fn makeHtml(&self, content: &str) -> String {
        let mut options = Options::empty();

        options.insert(Options::ENABLE_FOOTNOTES);
        if self.options.contains_key("strikethrough") && self.options["strikethrough"].is_boolean() && self.options["strikethrough"].as_bool().unwrap() {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if self.options.contains_key("tables") && self.options["tables"].is_boolean() && self.options["tables"].as_bool().unwrap() {
            options.insert(Options::ENABLE_TABLES);
        }
        if self.options.contains_key("tasklists") && self.options["tasklists"].is_boolean() && self.options["tasklists"].as_bool().unwrap() {
            options.insert(Options::ENABLE_TASKLISTS);
        }

        let parser = Parser::new_ext(content, options);

        let mut heading_level: Option<i64> = None;

        let parser = parser.filter_map(| event |
            match event {
                Event::Start(Tag::Heading(level @1..=6)) => {
                    if self.options.contains_key("noHeaderId") && self.options["noHeaderId"].is_boolean() && self.options["noHeaderId"].as_bool().unwrap() {
                        return Some(Event::Start(Tag::Heading(level)))
                    }
                    heading_level = Some(level as i64);
                    if self.options.contains_key("headerLevelStart") && self.options["headerLevelStart"].is_number() {
                        heading_level = Some(self.options["headerLevelStart"].as_i64().unwrap() + level as i64 - 1);
                    }
                    None
                },

                Event::Text(text) => {
                    if heading_level.is_some() {
                        //val.trim().toLowerCase().replace(/[^\w\- ]+/g, ' ').replace(/\s+/g, '-').replace(/\-+$/, '');
                        let anchor = text.clone().into_string().trim().to_lowercase().replace(" ", "-");
                        return Event::Html(CowStr::from(format!("<h{} id=\"{}\">{}</h{}>", heading_level.unwrap(), anchor, text, heading_level.unwrap()))).into()
                    }
                    Some(Event::Text(text))
                },

                Event::End(Tag::Heading(level)) => {
                    if self.options.contains_key("noHeaderId") && self.options["noHeaderId"].is_boolean() && self.options["noHeaderId"].as_bool().unwrap() {
                        return Some(Event::End(Tag::Heading(level)))
                    }
                    heading_level = None;
                    Some(Event::SoftBreak)
                },

                Event::Start(Tag::Strong) => {
                    if self.options.contains_key("literalMidWordAsterisks") && self.options["literalMidWordAsterisks"].is_boolean() && self.options["literalMidWordAsterisks"].as_bool().unwrap() {
                        return Some(Event::Text(CowStr::from("**")))
                    }
                    Some(Event::Start(Tag::Strong))
                },

                Event::End(Tag::Strong) => {
                    if self.options.contains_key("literalMidWordAsterisks") && self.options["literalMidWordAsterisks"].is_boolean() && self.options["literalMidWordAsterisks"].as_bool().unwrap() {
                        return Some(Event::Text(CowStr::from("**")))
                    }
                    Some(Event::End(Tag::Strong))
                },

                Event::SoftBreak => {
                    if self.options.contains_key("simpleLineBreaks") && self.options["simpleLineBreaks"].is_boolean() && self.options["simpleLineBreaks"].as_bool().unwrap() {
                        return Some(Event::HardBreak)
                    }
                    Some(Event::SoftBreak)
                },

                _ => Some(event),
            }
        );

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}