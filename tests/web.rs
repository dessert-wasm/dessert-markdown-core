//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use dessert_markdown_core::make_html;

use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

fn test(content: &str, result: &str) {
    assert_eq!(make_html(content, &JsValue::undefined()), result);
}

fn test_with_options(content: &str, result: &str, options: &JsValue) {
    assert_eq!(make_html(content, options), result);
}

#[wasm_bindgen_test]
fn tabs()
{
    test("\tfoo\tbaz\t\tbim\n", "<pre><code>foo\tbaz\t\tbim\n</code></pre>\n");
    test("  \tfoo\tbaz\t\tbim\n", "<pre><code>foo\tbaz\t\tbim\n</code></pre>\n");
    test("    a\ta\n    ὐ\ta\n", "<pre><code>a\ta\nὐ\ta\n</code></pre>\n");
    test("  - foo\n\n\tbar\n", "<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n</li>\n</ul>\n");
    test(">\t\tfoo\n", "<blockquote>\n<pre><code>  foo\n</code></pre>\n</blockquote>\n");
    test("#\tFoo\n", "<h1 id=\"foo\">Foo</h1>\n");
    test("*\t*\t*\t\n", "<hr />\n");
}

#[wasm_bindgen_test]
fn precedence()
{
    test("- `one\n- two`\n", "<ul>\n<li>`one</li>\n<li>two`</li>\n</ul>\n");
}

#[wasm_bindgen_test]
fn breaks() {
    test("***\n\n---\n\n___\n\n", "<hr />\n<hr />\n<hr />\n");
    test(" ***\n  ***\n   ***\n", "<hr />\n<hr />\n<hr />\n");
    test("    ***\n", "<pre><code>***\n</code></pre>\n");
    test("_____________________________________\n", "<hr />\n");
}

#[wasm_bindgen_test]
fn atx_headings() {
    let options = json!({"noHeaderId": true});
    let options = JsValue::from_serde(&options).unwrap();

    test_with_options("# foo\n## foo\n### foo\n#### foo\n##### foo\n###### foo\n", "<h1>foo</h1>\n<h2>foo</h2>\n<h3>foo</h3>\n<h4>foo</h4>\n<h5>foo</h5>\n<h6>foo</h6>\n", &options);
    //test_with_options("#5 bolt\n\n#hashtag\n", "<p>#5 bolt</p>\n<p>#hashtag</p>\n", &options);
    test_with_options(" ### foo\n  ## foo\n   # foo\n","<h3>foo</h3>\n<h2>foo</h2>\n<h1>foo</h1>\n", &options);
    //test_with_options("Foo *bar*\n=========\n\nFoo *bar*\n---------\n", "<h1>Foo <em>bar</em></h1>\n<h2>Foo <em>bar</em></h2>\n", &options);
}