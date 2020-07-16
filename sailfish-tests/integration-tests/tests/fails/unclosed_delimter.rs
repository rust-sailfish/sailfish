use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "unclosed_delimiter.stpl")]
struct UnclosedDelimiter {
    content: String,
}

fn main() {
    println!(
        "{}",
        UnclosedDelimiter {
            content: String::from("Hello, world!")
        }
        .render_once()
        .unwrap()
    )
}
