mod common;

use common::{stderr, stdout, Sandbox};

#[test]
fn undo_restores_the_original_layout() {
    let sb = Sandbox::new();
    sb.write("a.txt", "A");
    sb.write("b.png", "B");
    sb.sortr_here(&[]);
    assert!(!sb.exists("a.txt"));

    let out = sb.sortr(&["undo", sb.path().to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(sb.read("a.txt"), "A");
    assert_eq!(sb.read("b.png"), "B");
    assert!(!sb.exists("txt_folder"));
    assert!(!sb.exists("png_folder"));
    assert!(stdout(&out).contains("Restored 2 file(s)"));
}

#[test]
fn undo_with_no_history_exits_with_code_2() {
    let sb = Sandbox::new();
    let out = sb.sortr(&["undo", sb.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("nothing to undo"));
}

#[test]
fn undo_reports_files_that_cannot_be_restored() {
    let sb = Sandbox::new();
    sb.write("a.txt", "A");
    sb.sortr_here(&[]);
    sb.write("a.txt", "someone else's file");

    let out = sb.sortr(&["undo", sb.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(sb.read("a.txt"), "someone else's file");
    assert_eq!(sb.read("txt_folder/a.txt"), "A");
}
