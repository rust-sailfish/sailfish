use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
struct NoTemplate {
    var: usize
}

fn main() {
    println!("{}", NoTemplate { var: 1996 }.render_once().unwrap());
}
