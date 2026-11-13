mod common;

use common::{stderr, Sandbox};

#[test]
fn config_rules_choose_category_folders() {
    let sb = Sandbox::new();
    let cfg = sb.write(
        "rules/sortr.toml",
        "[categories]\nimages = [\"png\"]\ndocs = [\"txt\"]\n",
    );
    // Keep the rules file out of the directory being organized.
    let target = tempfile::tempdir().unwrap();
    std::fs::write(target.path().join("a.PNG"), "").unwrap();
    std::fs::write(target.path().join("b.txt"), "").unwrap();
    std::fs::write(target.path().join("c.zip"), "").unwrap();

    let out = sb.sortr(&[
        "--config",
        cfg.to_str().unwrap(),
        target.path().to_str().unwrap(),
    ]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert!(target.path().join("images/a.PNG").exists());
    assert!(target.path().join("docs/b.txt").exists());
    assert!(target.path().join("zip_folder/c.zip").exists());
}

#[test]
fn invalid_config_exits_with_code_2_and_moves_nothing() {
    let sb = Sandbox::new();
    let cfg = sb.write("rules/bad.toml", "[categories]\n\"../evil\" = [\"txt\"]\n");
    let target = tempfile::tempdir().unwrap();
    std::fs::write(target.path().join("a.txt"), "").unwrap();

    let out = sb.sortr(&["-c", cfg.to_str().unwrap(), target.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(2));
    assert!(target.path().join("a.txt").exists());
}

#[test]
fn missing_explicit_config_is_an_error() {
    let sb = Sandbox::new();
    sb.write("a.txt", "");
    let out = sb.sortr_here(&["--config", "/definitely/not/here.toml"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(sb.exists("a.txt"));
}
