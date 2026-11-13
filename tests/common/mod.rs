#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// A throwaway directory plus a way to run the real `sortr` binary against it.
/// HOME/XDG_CONFIG_HOME point at an empty dir so a developer's own config
/// file can never influence a test.
pub struct Sandbox {
    pub root: TempDir,
    home: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
            home: tempfile::tempdir().unwrap(),
        }
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }

    pub fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path().join(rel)).unwrap()
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.path().join(rel).exists()
    }

    /// Runs `sortr <args...>`.
    pub fn sortr(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sortr"))
            .args(args)
            .env("HOME", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path())
            .env("APPDATA", self.home.path())
            .output()
            .unwrap()
    }

    /// Runs `sortr [args...] <sandbox dir>`.
    pub fn sortr_here(&self, args: &[&str]) -> Output {
        let root = self.path().to_str().unwrap();
        let mut all = args.to_vec();
        all.push(root);
        self.sortr(&all)
    }
}

pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
