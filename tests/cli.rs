use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

const POLICY: &str = r#"
version = 1
max_files = 1
max_added_lines = 2
max_removed_lines = 1
allowed_paths = ["src/**"]
denied_paths = ["src/generated/**"]
"#;

const PATCH: &str = r"diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1 @@
-old
+new
";

#[test]
fn json_receipt_passes() {
    let directory = tempfile::tempdir().expect("temp dir");
    let policy = directory.path().join("policy.toml");
    let patch = directory.path().join("change.diff");
    fs::write(&policy, POLICY).expect("write policy");
    fs::write(&patch, PATCH).expect("write patch");

    let output = Command::new(env!("CARGO_BIN_EXE_patch-cage"))
        .args([
            "check",
            "--policy",
            policy.to_str().expect("utf8 path"),
            "--patch",
            patch.to_str().expect("utf8 path"),
            "--json",
        ])
        .output()
        .expect("run patch-cage");
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(report["passed"], true);
    assert_eq!(report["summary"]["files"], 1);
}

#[test]
fn stdin_violation_exits_two() {
    let directory = tempfile::tempdir().expect("temp dir");
    let policy = directory.path().join("policy.toml");
    fs::write(&policy, POLICY).expect("write policy");
    let denied = PATCH.replace("src/lib.rs", "src/generated/lib.rs");

    let mut child = Command::new(env!("CARGO_BIN_EXE_patch-cage"))
        .args([
            "check",
            "--policy",
            policy.to_str().expect("utf8 path"),
            "--patch",
            "-",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn patch-cage");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(denied.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for patch-cage");
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(report["passed"], false);
    assert_eq!(report["findings"][0]["code"], "path_denied");
}
