use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "missing_semicolon.stpl")]
struct MissingSemicolon {}

fn main() {
    println!("{}", (MissingSemicolon {}).render_once().unwrap());
}
