mod common;

use common::{stdout, Sandbox};

#[test]
fn dry_run_changes_nothing_and_lists_moves() {
    let sb = Sandbox::new();
    sb.write("a.txt", "A");

    let out = sb.sortr_here(&["--dry-run"]);

    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("a.txt ->"), "{text}");
    assert!(text.contains("nothing was changed"));
    assert!(sb.exists("a.txt"));
    assert!(!sb.exists("txt_folder"));
    assert!(!sb.exists(".sortr"), "dry run must not write a journal");
}
