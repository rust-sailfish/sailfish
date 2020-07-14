use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "foo.stpl", escape=1)]
struct InvalidOptionValue {
    name: String
}

fn main() {
    println!("{}", InvalidOptionValue { name: "Hanako".to_owned() }.render_once().unwrap());
}
