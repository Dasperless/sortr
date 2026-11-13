mod common;

use common::{stderr, stdout, Sandbox};

#[test]
fn organizes_files_by_extension() {
    let sb = Sandbox::new();
    sb.write("a.txt", "A");
    sb.write("photo.PNG", "P");
    sb.write("Makefile", "M");

    let out = sb.sortr_here(&[]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(sb.read("txt_folder/a.txt"), "A");
    assert_eq!(sb.read("png_folder/photo.PNG"), "P");
    assert_eq!(sb.read("no_extension/Makefile"), "M");
    assert!(!sb.exists("a.txt"));
    assert!(stdout(&out).contains("Moved 3 file(s)"));
}

#[test]
fn second_run_is_a_noop() {
    let sb = Sandbox::new();
    sb.write("a.txt", "A");
    sb.sortr_here(&[]);

    let out = sb.sortr_here(&[]);

    assert!(out.status.success());
    assert!(stdout(&out).contains("Moved 0 file(s)"));
    assert_eq!(sb.read("txt_folder/a.txt"), "A");
}

#[test]
fn existing_files_are_never_overwritten() {
    let sb = Sandbox::new();
    sb.write("txt_folder/a.txt", "old");
    sb.write("a.txt", "new");

    let out = sb.sortr_here(&[]);

    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(sb.read("txt_folder/a.txt"), "old");
    assert_eq!(sb.read("txt_folder/a (1).txt"), "new");
}

#[test]
fn subdirectories_are_ignored_unless_recursive() {
    let sb = Sandbox::new();
    sb.write("sub/deep.txt", "D");

    sb.sortr_here(&[]);
    assert!(sb.exists("sub/deep.txt"));

    let out = sb.sortr_here(&["--recursive"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(sb.read("txt_folder/deep.txt"), "D");
}

#[test]
fn hidden_files_are_left_alone() {
    let sb = Sandbox::new();
    sb.write(".env", "secret");
    sb.write("a.txt", "A");

    sb.sortr_here(&[]);

    assert!(sb.exists(".env"));
}

#[test]
fn missing_directory_exits_with_code_2() {
    let sb = Sandbox::new();
    let out = sb.sortr(&[sb.path().join("nope").to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("cannot read"));
}
