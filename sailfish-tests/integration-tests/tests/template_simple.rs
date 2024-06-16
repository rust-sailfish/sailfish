extern crate sailfish_macros;

use integration_tests::assert_string_eq;
use sailfish::runtime::RenderResult;
use sailfish::TemplateSimple;
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
fn assert_render_simple<T: TemplateSimple>(name: &str, template: T) {
    assert_render_result(name, template.render_once());
}
trait ConflictWithSailFishRender {
    fn render() {}
}

impl ConflictWithSailFishRender for u8 {}
impl ConflictWithSailFishRender for u16 {}

#[derive(TemplateSimple)]
#[template(path = "empty.stpl")]
struct Empty {}

#[test]
fn empty() {
    assert_render_simple("empty", Empty {});
}

#[derive(TemplateSimple)]
#[template(path = "noescape_s.stpl")]
struct Noescape<'a> {
    raw: &'a str,
}

#[test]
fn noescape() {
    assert_render_simple(
        "noescape",
        Noescape {
            raw: "<h1>Hello, World!</h1>",
        },
    );
}

#[derive(TemplateSimple)]
#[template(path = "json_s.stpl")]
struct Json {
    name: String,
    value: u16,
}

#[test]
fn json() {
    assert_render_simple(
        "json",
        Json {
            name: String::from("Taro"),
            value: 16,
        },
    );
}

#[derive(TemplateSimple)]
#[template(path = "custom_delimiter.stpl")]
#[template(delimiter = '🍣')]
struct CustomDelimiter;

#[test]
fn custom_delimiter() {
    assert_render_simple("custom_delimiter", CustomDelimiter);
}

#[derive(TemplateSimple)]
#[template(path = "include_s.stpl")]
struct Include<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include() {
    assert_render_simple(
        "include",
        Include {
            strs: &["foo", "bar"],
        },
    );
}

#[derive(TemplateSimple)]
#[template(path = "continue_break.stpl", rm_whitespace = true)]
struct ContinueBreak;

#[test]
fn continue_break() {
    assert_render_simple("continue_break", ContinueBreak);
}

#[derive(TemplateSimple)]
#[template(path = "techempower_s.stpl", rm_whitespace = true)]
struct Techempower {
    items: Vec<Fortune>,
}

struct Fortune {
    id: i32,
    message: &'static str,
}

#[test]
fn test_techempower() {
    let items = vec![
        Fortune {
            id: 0,
            message: "Additional fortune added at request time.",
        },
        Fortune {
            id: 1,
            message: "fortune: No such file or directory",
        },
        Fortune {
            id: 2,
            message: "A computer scientist is someone who fixes things that aren't broken.",
        },
        Fortune {
            id: 3,
            message: "After enough decimal places, nobody gives a damn.",
        },
        Fortune {
            id: 4,
            message: "A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1",
        },
        Fortune {
            id: 5,
            message: "A computer program does what you tell it to do, not what you want it to do.",
        },
        Fortune {
            id: 6,
            message: "Emacs is a nice operating system, but I prefer UNIX. — Tom Christaensen",
        },
        Fortune {
            id: 7,
            message: "Any program that runs right is obsolete.",
        },
        Fortune {
            id: 8,
            message: "A list is only as strong as its weakest link. — Donald Knuth",
        },
        Fortune {
            id: 9,
            message: "Feature: A bug with seniority.",
        },
        Fortune {
            id: 10,
            message: "Computers make very fast, very accurate mistakes.",
        },
        Fortune {
            id: 11,
            message: "<script>alert(\"This should not be displayed in a browser alert box.\");</script>",
        },
        Fortune {
            id: 12,
            message: "フレームワークのベンチマーク",
        },
    ];
    assert_render_simple("techempower", Techempower { items });
}

#[derive(TemplateSimple)]
#[template(path = "rm_whitespace_s.stpl")]
#[template(rm_whitespace = true)]
struct RmWhitespace<'a, 'b> {
    messages: &'a [&'b str],
}

#[test]
fn test_rm_whitespace() {
    assert_render_simple(
        "rm_whitespace",
        RmWhitespace {
            messages: &["foo", "bar"],
        },
    );
}

#[derive(TemplateSimple)]
#[template(path = "comment.stpl")]
struct Comment {}

#[test]
fn test_comment() {
    assert_render_simple("comment", Comment {})
}

#[derive(TemplateSimple)]
#[template(path = "rust_macro_s.stpl", rm_whitespace = true)]
struct RustMacro {
    value: Option<i32>,
}

#[test]
fn test_rust_macro() {
    assert_render_simple("rust_macro", RustMacro { value: Some(10) });
}

#[derive(TemplateSimple)]
#[template(path = "formatting.stpl", escape = false)]
struct Formatting;

#[test]
fn test_formatting() {
    assert_render_simple("formatting", Formatting);
}

#[derive(TemplateSimple)]
#[template(path = "filter_s.stpl")]
struct Filter<'a> {
    message: &'a str,
}

#[test]
fn test_filter() {
    assert_render_simple("filter", Filter { message: "hello" });
}

#[derive(TemplateSimple)]
#[template(path = "filter2.stpl")]
struct Filter2;

#[test]
fn test_filter2() {
    assert_render_simple("filter2", Filter2);
}

#[derive(TemplateSimple)]
#[template(path = "truncate_filter.stpl")]
struct TruncateFilter;

#[test]
fn test_truncate_filter() {
    assert_render_simple("truncate_filter", TruncateFilter);
}
#[derive(TemplateSimple)]
#[template(path = "json_filter_s.stpl")]
struct JsonFilter {
    data: serde_json::Value,
}

#[test]
fn test_json_filter() {
    let data = serde_json::json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    assert_render_simple("json_filter", JsonFilter { data });
}

#[cfg(unix)]
mod unix {
    use super::*;

    #[derive(TemplateSimple)]
    #[template(path = "include_nest_s.stpl")]
    struct IncludeNest<'a> {
        s: &'a str,
    }

    #[test]
    fn test_include_nest() {
        assert_render_simple("include_nest_s", IncludeNest { s: "foo" });
    }

    #[derive(TemplateSimple)]
    #[template(path = "include_rust_s.stpl")]
    struct IncludeRust {
        value: usize,
    }

    #[test]
    fn test_include_rust() {
        assert_render_simple("include_rust", IncludeRust { value: 58 });
    }
}
