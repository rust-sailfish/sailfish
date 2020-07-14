#[macro_use]
extern crate afl;

use sailfish::runtime as sf;
use sf::Render;

fn main() {
    fuzz!(|data: &[u8]| {
        // HTML escaping
        if let Ok(feed) = std::str::from_utf8(data) {
            let mut buf = sf::Buffer::new();
            let _ = feed.render_escaped(&mut buf);
        }
    });
}
