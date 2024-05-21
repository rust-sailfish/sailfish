use sailfish::Template;

#[derive(Template)]
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
    println!("{}", ctx.render().unwrap());
}
