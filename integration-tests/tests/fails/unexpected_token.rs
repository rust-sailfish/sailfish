use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

struct Content<'a> {
    id: u32,
    address: &'a str,
    phone_number: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "unexpected_token.stpl")]
#[template(escape = false)]
struct UnexpectedToken<'a> {
    name: &'a str,
    content: Content<'a>
}

fn main() {
    println!(
        "{}",
        UnclosedToken {
            name: "Taro",
            content: Content {
                id: 1,
                address: "oooo-xxxx",
                phone_number: "000-000-0000"
            }
        }
        .render_once()
        .unwrap()
    )
}
