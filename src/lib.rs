mod utils;

use wasm_bindgen::prelude::*;

#[macro_use]
extern crate lazy_static;
extern crate serde_derive;
use utils::*;

use js_sys::RegExp;
use std::sync::Mutex;
use serde_json::{json, Value};
use pulldown_cmark::{Parser, Options, html, Event, Tag, CowStr};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref OPTIONS: Mutex<Value> = {
        let m = json!({
            "customizedHeaderId": false,
            "emoji": false,
            "ghCompatibleHeaderId": false,
            "ghMentions": false,
            "ghMentionsLink": "https://github.com/{u}",
            "headerLevelStart": 1,
            "literalMidWordAsterisks": false,
            "noHeaderId": false,
            "openLinksInNewWindow": false,
            "prefixHeaderId": false,
            "rawHeaderId": false,
            "rawPrefixHeaderId": false,
            "requireSpaceBeforeHeadingText": false,
            "simpleLineBreaks": false,
            "strikethrough": false,
            "tables": false,
            "tasklists": false
        });
        Mutex::new(m)
    };
}

#[wasm_bindgen]
pub fn get_option(key: &str) -> JsValue {
    JsValue::from_serde(&OPTIONS.lock().unwrap()[key]).unwrap()
}

#[wasm_bindgen]
pub fn get_options() -> JsValue {
    JsValue::from_serde(&OPTIONS.lock().unwrap().as_object().unwrap()).unwrap()
}

#[wasm_bindgen]
pub fn set_option(key: String, value: JsValue) {
    OPTIONS.lock().unwrap()[key] = value.into_serde().unwrap();
}

fn add_link_to_mentions(text: &str, options: &serde_json::Value) -> String {
    let re = RegExp::new(r"\@[a-zA-Z][0-9a-zA-Z_]*", "gi");
    let mut ret = String::from(text);

    let mut result = re.exec(text);

    while result.is_some() {
        let mut v = vec![];
        result.unwrap().for_each(&mut |x, _, _| v.push(x));

        let href = if parse_bool(&options["openLinksInNewWindow"]) {
            "<a href=\"{link}\" rel=\"noopener noreferrer\" target=\"_blank\">{mention}</a>"
        } else {
            "<a href=\"{link}\">{mention}</a>"
        };
        let new_link = options["ghMentionsLink"].as_str().unwrap();

        for mention in v {
            let m = mention.as_string().unwrap();
            let complete_href = href.replace("{link}", new_link).replace("{u}", m.split_at(1).1).replace("{mention}", m.as_str());

            ret = ret.replace(m.as_str(), complete_href.as_str());
        }
        result = re.exec(text);
    }
    ret
}

fn handle_id(text: &mut CowStr, options: &serde_json::Value) -> String {
    let mut id = text.clone().into_string();

    if parse_bool(&options["customizedHeaderId"]) {
        let result = RegExp::new(r"\s?\{([^{]+?)}\s*$", "gm").exec(text);

        if result.is_some() {
            let arr = result.unwrap();
            *text = CowStr::from(text.replace(arr.get(0).as_string().unwrap().as_str(), ""));
            id = arr.get(1).as_string().unwrap();
        }
    }

    if parse_bool(&options["noHeaderId"]) {
        return String::new();
    }

    let prefix = if parse_bool(&options["prefixHeaderId"]) && options["prefixHeaderId"].is_boolean() {
        "section-"
    } else if parse_bool(&options["prefixHeaderId"]) && options["prefixHeaderId"].is_string() {
        options["prefixHeaderId"].as_str().unwrap()
    } else {
        ""
    };

    let mut title = if !parse_bool(&options["rawPrefixHeaderId"]) {
        format!("{}{}", prefix, id)
    } else {
        id
    };

    title = if parse_bool(&options["ghCompatibleHeaderId"]) {
        let new_title = title.replace(" ", "-");
        regex_replace(new_title.as_str(), "[&+$,/:;=?@\"#{}|^¨~[\\]`\\*)(%.!'<>]", "", "g").to_lowercase()
    } else if parse_bool(&options["rawHeaderId"]) {
        regex_replace(title.as_str(), "[\"\' ]", "-", "g").to_lowercase()
    } else {
        regex_replace(title.as_str(), r"[^\w]", "", "g").to_lowercase()
    };

    if parse_bool(&options["rawPrefixHeaderId"]) {
        title = format!("{}{}", prefix, title);
    };

    format!(" id=\"{}\"", title)
}

fn handle_header(text: &mut CowStr, hlevel: u32, options: &serde_json::Value) -> String {
    let hid = handle_id(text, options);

    format!("<h{}{}>{}</h{}>", hlevel, hid, text, hlevel)
}

#[wasm_bindgen]
pub fn make_html(content: &str, options: &JsValue) -> String {
    let options: serde_json::Value = options.into_serde().unwrap();
    let mut cmark_options = Options::empty();

    cmark_options.insert(Options::ENABLE_FOOTNOTES);
    if parse_bool(&options["strikethrough"]) {
        cmark_options.insert(Options::ENABLE_STRIKETHROUGH);
    }
    if parse_bool(&options["tables"]) {
        cmark_options.insert(Options::ENABLE_TABLES);
    }
    if parse_bool(&options["tasklists"]) {
        cmark_options.insert(Options::ENABLE_TASKLISTS);
    }

    let mut parse_event : Option<Event> = None;
    let parser = Parser::new_ext(content, cmark_options);

    let parser = parser.filter_map(|event|
        match event {
            Event::Start(Tag::Heading(level @ 1..=6)) => {
                let header_level_start = parse_int(&options["headerLevelStart"]);

                parse_event = if header_level_start.is_some() {
                    Some(Event::Start(Tag::Heading(header_level_start.unwrap() as u32 + level - 1)))
                } else {
                    Some(Event::Start(Tag::Heading(level)))
                };
                None
            }

            Event::Start(Tag::Link(link_type, destination, url)) => {
                if parse_bool(&options["openLinksInNewWindow"]) {
                    parse_event = Some(Event::Start(Tag::Link(link_type, destination, url)));
                    None
                } else {
                    Some(Event::Start(Tag::Link(link_type, destination, url)))
                }
            }

            Event::Text(mut text) => {
                let mut html = false;

                match &parse_event {
                    Some(Event::Start(Tag::Heading(level))) => {
                        text = CowStr::from(handle_header(&mut text, *level, &options));
                        html = true;
                    },
                    Some(Event::Start(Tag::Link(_, destination, _))) => {
                        text = CowStr::from(format!("<a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\">{}</a>", destination, text));
                        html = true;
                        parse_event = None;
                    },
                    None => {
                        if !parse_bool(&options["requireSpaceBeforeHeadingText"]) && text.contains("#") {
                            let count = text.chars().take_while(|x| x == &'#').count() as u32;
                            text = CowStr::from(text.replace("#", ""));

                            let level = match parse_int(&options["headerLevelStart"]) {
                                None => {
                                    count
                                },
                                Some(level) => {
                                    level as u32 + count - 1
                                },
                            };
                            text = CowStr::from(handle_header(&mut text, level, &options));
                            html = true;

                        }
                    },
                    Some(_) => (),
                }

                if parse_bool(&options["emoji"]) {
                    //https://github.com/showdownjs/showdown/wiki/Emojis
                    text = CowStr::from(gh_emoji::Replacer::new().replace_all(&text).into_owned());
                }

                if text.find("@").is_some() && parse_bool(&options["ghMentions"]) {
                    text = CowStr::from(add_link_to_mentions(&text, &options));
                    html = true;
                }

                if html {
                    Some(Event::Html(text))
                } else {
                    Some(Event::Text(text))
                }
            }

            Event::End(Tag::Heading(_)) => {
                parse_event = None;
                Some(Event::SoftBreak)
            }

            Event::Start(Tag::Strong) => {
                if parse_bool(&options["literalMidWordAsterisks"]) {
                    return Some(Event::Text(CowStr::from("**")));
                }
                Some(Event::Start(Tag::Strong))
            }

            Event::End(Tag::Strong) => {
                if parse_bool(&options["literalMidWordAsterisks"]) {
                    return Some(Event::Text(CowStr::from("**")));
                }
                Some(Event::End(Tag::Strong))
            }

            Event::SoftBreak => {
                if parse_bool(&options["simpleLineBreaks"]) {
                    return Some(Event::HardBreak);
                }
                Some(Event::SoftBreak)
            }
            _ => Some(event),
        }
    );

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}