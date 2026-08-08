//! End-to-end smoke: drives the real binary through the full loop on a copy
//! of the repo's example cases. Only depends on repo files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("opencase-smoke-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let e = entry.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            fs::copy(e.path(), to).unwrap();
        }
    }
}

fn opencase(cases: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_opencase"))
        .arg("--cases")
        .arg(cases)
        .args(args)
        .output()
        .unwrap()
}

fn ok(out: &Output, what: &str) {
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn full_loop_smoke() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let d = tmpdir("full-loop");
    copy_dir(&repo.join("cases"), &d.join("cases"));
    copy_dir(&repo.join("tests"), &d.join("tests"));
    let cases = d.join("cases");

    // validate the shipped examples
    let out = opencase(&cases, &["validate"]);
    ok(&out, "validate");
    assert!(String::from_utf8_lossy(&out.stdout).contains("0 problem(s)"));

    // writer adds a draft case
    fs::write(
        cases.join("checkout-success.md"),
        "---\nid: checkout-success\ntitle: Checkout succeeds\nstatus: draft\nmode: manual\nsource: PRD §3\n---\n\n## Steps\n\n1. Add to cart\n2. Check out\n\n## Expected\n\n- Order created\n",
    )
    .unwrap();

    // review gate: run on a draft is rejected
    let out = opencase(&cases, &["run", "checkout-success"]);
    assert!(!out.status.success(), "run on draft must be rejected");
    assert!(String::from_utf8_lossy(&out.stderr).contains("review gate"));

    // review lists the draft and approves it
    let out = opencase(&cases, &["review"]);
    ok(&out, "review list");
    assert!(String::from_utf8_lossy(&out.stdout).contains("checkout-success"));
    let out = opencase(&cases, &["review", "checkout-success", "--approve"]);
    ok(&out, "review approve");

    // run prints the prompt
    let out = opencase(&cases, &["run", "checkout-success"]);
    ok(&out, "run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("## Steps") && stdout.contains("checkout"),
        "run: {stdout}"
    );

    // record pass
    let out = opencase(
        &cases,
        &[
            "record",
            "checkout-success",
            "--result",
            "pass",
            "--commit",
            "abc",
        ],
    );
    ok(&out, "record pass");

    // record fail without category is rejected
    let out = opencase(&cases, &["record", "checkout-success", "--result", "fail"]);
    assert!(
        !out.status.success(),
        "fail without category must be rejected"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("--category"));

    // record fail with attribution
    let out = opencase(
        &cases,
        &[
            "record",
            "checkout-success",
            "--result",
            "fail",
            "--category",
            "test-bug",
            "--commit",
            "abc",
            "--note",
            "wrong expectation",
        ],
    );
    ok(&out, "record fail");

    // report shows the case and its last run; validate stays clean
    let out = opencase(&cases, &["report"]);
    ok(&out, "report");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("checkout-success"), "report: {stdout}");
    assert!(stdout.contains("fail 20"), "report: {stdout}");
    let out = opencase(&cases, &["validate"]);
    ok(&out, "validate after loop");
    assert!(String::from_utf8_lossy(&out.stdout).contains("0 problem(s)"));
}
