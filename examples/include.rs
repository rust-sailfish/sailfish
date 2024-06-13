use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "include.stpl")]
struct Include {
    title: String,
    name: String,
}

fn main() {
    let ctx = Include {
        title: "Website".to_owned(),
        name: "Hanako".to_owned(),
    };
    println!("{}", ctx.render_once().unwrap());
}
