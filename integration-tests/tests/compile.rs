// use std::fs;
// use std::path::{Path, PathBuf};
// 
// struct TempDir {
//     dir: PathBuf
// }
// 
// impl TempDir {
//     fn new(dir: &Path) {
//         fs::create_dir_all(dir);
//         Self { dir: dir.to_owned() }
//     }
// }
// 
// impl Frop for TempDir {
//     fn drop(&mut self) {
//         fs::remove_dir_all(&*self.dir);
//     }
// }

#[test]
fn compile_error() {
    std::env::set_var("SAILFISH_INTEGRATION_TESTS", "1");
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fails/*.rs");
}
