use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub delimiter: char,
    pub escape: bool,
    pub cache_dir: PathBuf,
    pub rm_whitespace: bool,
    #[doc(hidden)]
    pub _non_exhaustive: ()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            delimiter: '%',
            escape: true,
            cache_dir: Path::new(env!("OUT_DIR")).join("cache"),
            rm_whitespace: false,
            _non_exhaustive: ()
        }
    }
}

// TODO: Global configration file
