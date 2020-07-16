use sailfish_compiler::Config;
use std::path::Path;

#[test]
fn read_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
    let config = Config::search_file_and_read(&*path).unwrap();

    assert_eq!(config.delimiter, '%');
    assert_eq!(config.escape, true);
    assert_eq!(config.rm_whitespace, false);
    assert_eq!(config.template_dirs.len(), 1);
}
