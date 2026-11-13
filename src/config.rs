//! Configurable rules mapping file extensions to category folders.
//!
//! ```toml
//! [categories]
//! images = ["jpg", "jpeg", "png"]
//! docs = ["pdf", "txt"]
//! ```
//!
//! Extensions without a rule fall back to `<ext>_folder`.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::planner;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read config {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid config {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("invalid category name {0:?}: must be a plain folder name")]
    BadCategory(String),
    #[error("extension {ext:?} is listed in both {first:?} and {second:?}")]
    DuplicateExtension {
        ext: String,
        first: String,
        second: String,
    },
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    categories: BTreeMap<String, Vec<String>>,
}

/// Compiled rules: extension -> category folder.
#[derive(Debug, Default, Clone)]
pub struct Rules {
    by_extension: HashMap<String, String>,
}

impl Rules {
    /// Parses and validates rules from TOML text.
    pub fn parse(text: &str, path: &Path) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_categories(raw.categories)
    }

    /// Loads rules from a TOML file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text, path)
    }

    fn from_categories(categories: BTreeMap<String, Vec<String>>) -> Result<Self, ConfigError> {
        let mut by_extension: HashMap<String, String> = HashMap::new();
        for (category, extensions) in categories {
            if !is_plain_folder_name(&category) {
                return Err(ConfigError::BadCategory(category));
            }
            for ext in extensions {
                let ext = ext.trim_start_matches('.').to_lowercase();
                if let Some(first) = by_extension.insert(ext.clone(), category.clone()) {
                    return Err(ConfigError::DuplicateExtension {
                        ext,
                        first,
                        second: category,
                    });
                }
            }
        }
        Ok(Self { by_extension })
    }

    /// Category folder for a file name, falling back to the default naming.
    pub fn classify(&self, file_name: &OsStr) -> String {
        planner::extension_of(file_name)
            .and_then(|ext| self.by_extension.get(&ext).cloned())
            .unwrap_or_else(|| planner::default_category(file_name))
    }
}

/// Rejects names that could escape the target directory.
fn is_plain_folder_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\'])
        && !name.contains('\0')
}

/// `<config dir>/sortr/config.toml`, e.g. `~/.config/sortr/config.toml`.
pub fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sortr").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(text: &str) -> Result<Rules, ConfigError> {
        Rules::parse(text, Path::new("test.toml"))
    }

    #[test]
    fn maps_extensions_case_insensitively() {
        let r = rules("[categories]\nimages = [\"JPG\", \".png\"]").unwrap();
        assert_eq!(r.classify(OsStr::new("a.jpg")), "images");
        assert_eq!(r.classify(OsStr::new("b.PNG")), "images");
    }

    #[test]
    fn unmatched_files_use_default_naming() {
        let r = rules("[categories]\nimages = [\"png\"]").unwrap();
        assert_eq!(r.classify(OsStr::new("a.zip")), "zip_folder");
        assert_eq!(r.classify(OsStr::new("Makefile")), planner::NO_EXTENSION);
    }

    #[test]
    fn empty_config_is_valid() {
        let r = rules("").unwrap();
        assert_eq!(r.classify(OsStr::new("a.txt")), "txt_folder");
    }

    #[test]
    fn duplicate_extension_is_rejected() {
        let err = rules("[categories]\na = [\"x\"]\nb = [\"x\"]").unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateExtension { .. }));
    }

    #[test]
    fn path_like_category_names_are_rejected() {
        for bad in ["../evil", "a/b", "a\\\\b", ".."] {
            let text = format!("[categories]\n\"{bad}\" = [\"x\"]");
            assert!(
                matches!(rules(&text), Err(ConfigError::BadCategory(_))),
                "{bad} should be rejected"
            );
        }
    }

    #[test]
    fn malformed_toml_reports_the_path() {
        let err = rules("[categories\n").unwrap_err();
        assert!(err.to_string().contains("test.toml"));
    }

    #[test]
    fn load_reports_missing_file() {
        let err = Rules::load(Path::new("/definitely/not/here.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::Read { .. }));
    }
}
