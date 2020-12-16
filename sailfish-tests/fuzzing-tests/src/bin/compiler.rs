#[macro_use]
extern crate afl;

use sailfish_compiler::Compiler;

fn main() {
    fuzz!(|data: &[u8]| {
        let compiler = Compiler::default();
        let feed = data.iter().map(|&b| char::from(b)).collect::<String>();
        let _ = compiler.compile_str(&*feed);
    });
}
