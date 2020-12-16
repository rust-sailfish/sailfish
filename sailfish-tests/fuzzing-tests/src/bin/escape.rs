#[macro_use]
extern crate afl;

use sailfish::runtime as sf;
use sf::Render;

fn main() {
    fuzz!(|data: &[u8]| {
        // HTML escaping
        let mut buf = sf::Buffer::new();
        let feed = data.iter().map(|&b| char::from(b)).collect::<String>();
        let _ = feed.render_escaped(&mut buf);
    });
}
