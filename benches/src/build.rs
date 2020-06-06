use ructe::Ructe;
use std::env;
use std::path::PathBuf;

fn main() {
    let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("templates_ructe");
    Ructe::from_env()
        .unwrap()
        .compile_templates(&in_dir)
        .expect("compile templates");
}
