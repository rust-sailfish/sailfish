use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
#[template(patth = "foo.stpl")]
struct UnknownOption {
    name: String
}

fn main() {
    println!("{}", UnknownOption { name: "Hanako".to_owned() }.render_once().unwrap());
}
