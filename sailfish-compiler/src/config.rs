use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub delimiter: char,
    pub escape: bool,
    pub rm_whitespace: bool,
    pub template_dirs: Vec<PathBuf>,
    #[doc(hidden)]
    pub cache_dir: PathBuf,
    #[doc(hidden)]
    pub _non_exhaustive: (),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template_dirs: Vec::new(),
            delimiter: '%',
            escape: true,
            cache_dir: Path::new(env!("OUT_DIR")).join("cache"),
            rm_whitespace: false,
            _non_exhaustive: (),
        }
    }
}

#[cfg(feature = "config")]
mod imp {
    use std::fs;
    use yaml_rust::yaml::{Yaml, YamlLoader};

    use super::*;
    use crate::error::*;

    impl Config {
        pub fn search_file_and_read(base: &Path) -> Result<Config, Error> {
            // search config file
            let mut path = PathBuf::new();
            let mut config = Config::default();

            for component in base.iter() {
                path.push(component);
                path.push("sailfish.yml");

                if path.is_file() {
                    let config_file =
                        ConfigFile::read_from_file(&*path).map_err(|mut e| {
                            e.source_file = Some(path.to_owned());
                            e
                        })?;

                    if let Some(template_dirs) = config_file.template_dirs {
                        for template_dir in template_dirs.into_iter().rev() {
                            if template_dir.is_absolute() {
                                config.template_dirs.push(template_dir);
                            } else {
                                config
                                    .template_dirs
                                    .push(path.parent().unwrap().join(template_dir));
                            }
                        }
                    }

                    if let Some(delimiter) = config_file.delimiter {
                        config.delimiter = delimiter;
                    }

                    if let Some(escape) = config_file.escape {
                        config.escape = escape;
                    }

                    if let Some(rm_whitespace) = config_file.rm_whitespace {
                        config.rm_whitespace = rm_whitespace;
                    }
                }

                path.pop();
            }

            Ok(config)
        }
    }

    #[derive(Default)]
    struct ConfigFile {
        template_dirs: Option<Vec<PathBuf>>,
        delimiter: Option<char>,
        escape: Option<bool>,
        rm_whitespace: Option<bool>,
    }

    impl ConfigFile {
        fn read_from_file(path: &Path) -> Result<Self, Error> {
            let mut config = Self::default();
            let content = fs::read_to_string(path)
                .chain_err(|| format!("Failed to read configuration file {:?}", path))?;

            let entries = YamlLoader::load_from_str(&*content)
                .map_err(|e| ErrorKind::ConfigError(e.to_string()))?;
            drop(content);

            for entry in entries {
                config.visit_global(entry)?
            }

            Ok(config)
        }

        fn visit_global(&mut self, entry: Yaml) -> Result<(), Error> {
            let hash = entry.into_hash().ok_or_else(|| {
                ErrorKind::ConfigError("Invalid configuration format".to_owned())
            })?;

            for (k, v) in hash {
                match k {
                    Yaml::String(ref s) => match &**s {
                        "template_dir" => self.visit_template_dir(v)?,
                        "delimiter" => self.visit_delimiter(v)?,
                        "escape" => self.visit_escape(v)?,
                        "optimization" => self.visit_optimization(v)?,
                        _ => return Err(Self::error(format!("Unknown key ({})", s))),
                    },
                    _ => {
                        return Err(Self::error("Invalid configuration format"));
                    }
                }
            }

            Ok(())
        }

        fn visit_template_dir(&mut self, value: Yaml) -> Result<(), Error> {
            if self.template_dirs.is_some() {
                return Err(Self::error("Duplicate key (template_dir)"));
            }

            match value {
                Yaml::String(s) => self.template_dirs = Some(vec![PathBuf::from(s)]),
                Yaml::Array(v) => {
                    let mut template_dirs = Vec::new();
                    for e in v {
                        if let Yaml::String(s) = e {
                            template_dirs.push(PathBuf::from(s));
                        } else {
                            return Err(Self::error(
                                "Arguments of `template_dir` must be string",
                            ));
                        }
                    }
                    self.template_dirs = Some(template_dirs);
                }
                _ => {
                    return Err(Self::error(
                        "Arguments of `template_dir` must be string",
                    ));
                }
            }

            Ok(())
        }

        fn visit_delimiter(&mut self, value: Yaml) -> Result<(), Error> {
            if self.delimiter.is_some() {
                return Err(Self::error("Duplicate key (delimiter)"));
            }

            if let Yaml::String(s) = value {
                if s.chars().count() == 1 {
                    self.delimiter = Some(s.chars().next().unwrap());
                    Ok(())
                } else {
                    Err(Self::error("`escape` must be single character"))
                }
            } else {
                Err(Self::error("`escape` must be single character"))
            }
        }

        fn visit_escape(&mut self, value: Yaml) -> Result<(), Error> {
            if self.escape.is_some() {
                return Err(Self::error("Duplicate key (escape)"));
            }

            if let Yaml::Boolean(b) = value {
                self.escape = Some(b);
                Ok(())
            } else {
                Err(Self::error("`escape` must be boolean"))
            }
        }

        fn visit_optimization(&mut self, entry: Yaml) -> Result<(), Error> {
            let hash = entry.into_hash().ok_or_else(|| {
                ErrorKind::ConfigError("Invalid configuration format".to_owned())
            })?;

            for (k, v) in hash {
                match k {
                    Yaml::String(ref s) => match &**s {
                        "rm_whitespace" => self.visit_rm_whitespace(v)?,
                        _ => {
                            return Err(Self::error(format!(
                                "Unknown key (optimization.{})",
                                s
                            )));
                        }
                    },
                    _ => {
                        return Err(Self::error("Invalid configuration format"));
                    }
                }
            }

            Ok(())
        }

        fn visit_rm_whitespace(&mut self, value: Yaml) -> Result<(), Error> {
            if self.rm_whitespace.is_some() {
                return Err(Self::error("Duplicate key (rm_whitespace)"));
            }

            if let Yaml::Boolean(b) = value {
                self.rm_whitespace = Some(b);
                Ok(())
            } else {
                Err(Self::error("`rm_whitespace` must be boolean"))
            }
        }

        fn error<T: Into<String>>(msg: T) -> Error {
            make_error!(ErrorKind::ConfigError(msg.into()))
        }
    }
}
