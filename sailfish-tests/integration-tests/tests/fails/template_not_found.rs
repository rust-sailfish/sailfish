use sailfish::TemplateOnce;
use sailfish_macros::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "empty.stpl")]
struct ExistTemplate;

#[derive(TemplateOnce)]
#[template(path = "not_exist.stpl")]
struct NotExistTemplate {
    var: usize
}

fn main() {
    println!("{}", ExistTemplate.render_once().unwrap());
    println!("{}", NotExistTemplate { var: 1996 }.render_once().unwrap());
}
