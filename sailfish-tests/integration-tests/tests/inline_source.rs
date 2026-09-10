extern crate sailfish_macros;

use integration_tests::assert_string_eq;
use sailfish::{Template, TemplateSimple};

#[derive(TemplateSimple)]
#[template(source = "<div><%= name %></div>")]
struct Greet<'a> {
    name: &'a str,
}

#[test]
fn render_from_source() {
    let rendered = Greet { name: "World" }.render_once().unwrap();
    assert_string_eq!(&*rendered, "<div>World</div>");
}

#[derive(TemplateSimple)]
#[template(source = "<%= raw %>")]
struct Escaped<'a> {
    raw: &'a str,
}

#[test]
fn source_escapes_by_default() {
    let rendered = Escaped {
        raw: "<h1>Hello</h1>",
    }
    .render_once()
    .unwrap();
    assert_string_eq!(&*rendered, "&lt;h1&gt;Hello&lt;/h1&gt;");
}

#[derive(TemplateSimple)]
#[template(source = "<%= raw %>", escape = false)]
struct Noescape<'a> {
    raw: &'a str,
}

#[test]
fn source_respects_options() {
    let rendered = Noescape {
        raw: "<h1>Hello</h1>",
    }
    .render_once()
    .unwrap();
    assert_string_eq!(&*rendered, "<h1>Hello</h1>");
}

#[derive(TemplateSimple)]
#[template(source = "[ <% for i in 0..3 { %><%= i %><% } %> ]")]
struct Loop {}

#[test]
fn source_supports_control_flow() {
    let rendered = Loop {}.render_once().unwrap();
    assert_string_eq!(&*rendered, "[ 012 ]");
}

#[derive(Template)]
#[template(source = "Hi <%= self.name %>")]
struct Borrowed<'a> {
    name: &'a str,
}

#[test]
fn source_works_with_template_trait() {
    let t = Borrowed { name: "there" };
    assert_string_eq!(&*t.render().unwrap(), "Hi there");
}
