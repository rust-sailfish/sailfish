#[macro_use]
extern crate sailfish_macros;

use integration_tests::assert_string_eq;
use sailfish::runtime::RenderResult;
use sailfish::TemplateOnce;
use std::path::PathBuf;

fn assert_render_result(name: &str, result: RenderResult) {
    let mut output_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output_file.push("templates");
    output_file.push(name);
    output_file.set_extension("out");

    let mut expected = std::fs::read_to_string(output_file).unwrap();
    if expected.ends_with('\n') {
        expected.truncate(expected.len() - 1);
        if expected.ends_with('\r') {
            expected.truncate(expected.len() - 1);
        }
    }
    assert_string_eq!(&*result.unwrap(), &*expected);
}

#[inline]
fn assert_render<T: TemplateOnce>(name: &str, template: T) {
    assert_render_result(name, template.render_once());
}

#[derive(TemplateOnce)]
#[template(path = "empty.stpl")]
struct Empty {}

#[test]
fn empty() {
    assert_render("empty", Empty {});
}

#[derive(TemplateOnce)]
#[template(path = "noescape.stpl")]
struct Noescape<'a> {
    raw: &'a str,
}

#[test]
fn noescape() {
    assert_render(
        "noescape",
        Noescape {
            raw: "<h1>Hello, World!</h1>",
        },
    );
}

#[derive(TemplateOnce)]
#[template(path = "json.stpl")]
struct Json {
    name: String,
    value: u16,
}

#[test]
fn json() {
    assert_render(
        "json",
        Json {
            name: String::from("Taro"),
            value: 16,
        },
    );
}

#[derive(TemplateOnce)]
#[template(path = "custom_delimiter.stpl")]
#[template(delimiter = '🍣')]
struct CustomDelimiter;

#[test]
fn custom_delimiter() {
    assert_render("custom_delimiter", CustomDelimiter);
}

#[derive(TemplateOnce)]
#[template(path = "include.stpl")]
struct Include<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include() {
    assert_render("include", Include { strs: &["foo", "bar"] });
}

#[derive(TemplateOnce)]
#[template(path = "include-nest.stpl")]
struct IncludeNest<'a> {
    s: &'a str,
}

#[test]
fn test_include_nest() {
    assert_render("include-nest", IncludeNest { s: "foo" });
}
