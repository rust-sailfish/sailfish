use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "simple.stpl")]
struct Simple {
    messages: Vec<String>,
}

fn main() {
    let messages = vec![String::from("Message 1"), String::from("<Message 2>")];
    println!("{}", Simple { messages }.render_once().unwrap());
}
