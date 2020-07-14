#[macro_use]
extern crate afl;

use sailfish_compiler::Compiler;

fn main() {
    fuzz!(|data: &[u8]| {
        // HTML escaping
        if let Ok(feed) = std::str::from_utf8(data) {
            let compiler = Compiler::default();
            let _ = compiler.compile_str(feed);
        }
    });
}
