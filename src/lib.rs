//! Parse a conventional Git patch and check it against a remediation envelope.

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Component, Path};
use thiserror::Error;

/// Errors that prevent a patch from being evaluated safely.
#[derive(Debug, Error)]
pub enum CageError {
    #[error("policy TOML is invalid: {0}")]
    PolicyToml(#[from] toml::de::Error),
    #[error("unsupported policy version {0}; expected version 1")]
    PolicyVersion(u32),
    #[error("policy must contain at least one allowed_paths pattern")]
    MissingAllowedPaths,
    #[error("invalid glob pattern `{pattern}`: {message}")]
    InvalidGlob { pattern: String, message: String },
    #[error("patch contains no `diff --git` file records")]
    EmptyPatch,
    #[error("line {line}: {message}")]
    MalformedPatch { line: usize, message: String },
}

/// Declarative limits for an admitted patch.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)] // TOML keeps each operation independently reviewable.
pub struct Policy {
    pub version: u32,
    pub max_files: usize,
    pub max_added_lines: usize,
    pub max_removed_lines: usize,
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub denied_paths: Vec<String>,
    #[serde(default)]
    pub allow_new_files: bool,
    #[serde(default)]
    pub allow_deleted_files: bool,
    #[serde(default)]
    pub allow_renames: bool,
    #[serde(default)]
    pub allow_binary: bool,
    #[serde(default)]
    pub allow_mode_changes: bool,
}

impl Policy {
    /// Parse and validate a policy document.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed TOML, unknown fields, unsupported versions,
    /// missing allow patterns, or invalid glob syntax.
    pub fn from_toml(input: &str) -> Result<Self, CageError> {
        let policy: Self = toml::from_str(input)?;
        if policy.version != 1 {
            return Err(CageError::PolicyVersion(policy.version));
        }
        if policy.allowed_paths.is_empty() {
            return Err(CageError::MissingAllowedPaths);
        }
        compile_globs(&policy.allowed_paths)?;
        compile_globs(&policy.denied_paths)?;
        Ok(policy)
    }
}

/// How a file is changed by the patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
}

/// A parsed file-level patch record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileChange {
    pub path: String,
    pub old_path: String,
    pub kind: ChangeKind,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub binary: bool,
    pub mode_changed: bool,
}

/// Parsed patch totals and file records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub files: Vec<FileChange>,
    pub added_lines: usize,
    pub removed_lines: usize,
}

#[derive(Debug, Clone)]
struct FileBuilder {
    old_path: String,
    new_path: String,
    kind: ChangeKind,
    added_lines: usize,
    removed_lines: usize,
    binary: bool,
    mode_changed: bool,
    in_hunk: bool,
}

impl FileBuilder {
    fn finish(self) -> FileChange {
        let path = if self.kind == ChangeKind::Deleted {
            self.old_path.clone()
        } else {
            self.new_path.clone()
        };
        FileChange {
            path,
            old_path: self.old_path,
            kind: self.kind,
            added_lines: self.added_lines,
            removed_lines: self.removed_lines,
            binary: self.binary,
            mode_changed: self.mode_changed,
        }
    }
}

/// Parse a conventional `git diff` patch without touching the working tree.
///
/// # Errors
///
/// Returns an error when no file records exist, a header is ambiguous, or patch
/// content appears outside a recognized file record.
pub fn parse_patch(input: &str) -> Result<Patch, CageError> {
    let mut files = Vec::new();
    let mut current: Option<FileBuilder> = None;

    for (index, line) in input.lines().enumerate() {
        let line_number = index + 1;
        if let Some(header) = line.strip_prefix("diff --git ") {
            if let Some(file) = current.take() {
                files.push(file.finish());
            }
            current = Some(parse_header(header, line_number)?);
            continue;
        }

        let Some(file) = current.as_mut() else {
            if line.trim().is_empty() || line.starts_with("From ") || line.starts_with("Subject: ")
            {
                continue;
            }
            return Err(CageError::MalformedPatch {
                line: line_number,
                message: "content appears before the first `diff --git` record".to_owned(),
            });
        };

        if line.starts_with("new file mode ") {
            file.kind = ChangeKind::Added;
            file.in_hunk = false;
        } else if line.starts_with("deleted file mode ") {
            file.kind = ChangeKind::Deleted;
            file.in_hunk = false;
        } else if let Some(path) = line.strip_prefix("rename from ") {
            file.kind = ChangeKind::Renamed;
            file.old_path = parse_metadata_path(path, line_number)?;
            file.in_hunk = false;
        } else if let Some(path) = line.strip_prefix("rename to ") {
            file.kind = ChangeKind::Renamed;
            file.new_path = parse_metadata_path(path, line_number)?;
            file.in_hunk = false;
        } else if line.starts_with("old mode ") || line.starts_with("new mode ") {
            file.mode_changed = true;
            file.in_hunk = false;
        } else if line == "GIT binary patch" || line.starts_with("Binary files ") {
            file.binary = true;
            file.in_hunk = false;
        } else if line.starts_with("@@ ") {
            file.in_hunk = true;
        } else if file.in_hunk && line.starts_with('+') && !line.starts_with("+++") {
            file.added_lines += 1;
        } else if file.in_hunk && line.starts_with('-') && !line.starts_with("---") {
            file.removed_lines += 1;
        }
    }

    if let Some(file) = current {
        files.push(file.finish());
    }
    if files.is_empty() {
        return Err(CageError::EmptyPatch);
    }

    let added_lines = files.iter().map(|file| file.added_lines).sum();
    let removed_lines = files.iter().map(|file| file.removed_lines).sum();
    Ok(Patch {
        files,
        added_lines,
        removed_lines,
    })
}

fn parse_header(header: &str, line: usize) -> Result<FileBuilder, CageError> {
    let parts: Vec<_> = header.split_whitespace().collect();
    if parts.len() != 2 || parts.iter().any(|part| part.starts_with('"')) {
        return Err(CageError::MalformedPatch {
            line,
            message: "expected an unquoted `a/path b/path` header; quoted paths are not supported"
                .to_owned(),
        });
    }
    let old_path = strip_git_prefix(parts[0], "a/", line)?;
    let new_path = strip_git_prefix(parts[1], "b/", line)?;
    Ok(FileBuilder {
        old_path,
        new_path,
        kind: ChangeKind::Modified,
        added_lines: 0,
        removed_lines: 0,
        binary: false,
        mode_changed: false,
        in_hunk: false,
    })
}

fn strip_git_prefix(value: &str, prefix: &str, line: usize) -> Result<String, CageError> {
    let path = value
        .strip_prefix(prefix)
        .ok_or_else(|| CageError::MalformedPatch {
            line,
            message: format!("expected path prefix `{prefix}` in `{value}`"),
        })?;
    parse_metadata_path(path, line)
}

fn parse_metadata_path(value: &str, line: usize) -> Result<String, CageError> {
    if value.is_empty() || value.starts_with('"') || value.ends_with('"') {
        return Err(CageError::MalformedPatch {
            line,
            message: "empty or quoted paths are not supported".to_owned(),
        });
    }
    Ok(value.to_owned())
}

/// A stable policy finding suitable for CI automation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub code: String,
    pub path: Option<String>,
    pub message: String,
}

/// Compact totals included in the receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub files: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
}

/// Deterministic evaluation receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub passed: bool,
    pub policy_sha256: String,
    pub patch_sha256: String,
    pub summary: Summary,
    pub files: Vec<FileChange>,
    pub findings: Vec<Finding>,
}

/// Evaluate a parsed patch against the supplied policy.
///
/// # Errors
///
/// Returns an error only if a previously constructed policy contains an invalid
/// glob. Policy violations are returned as findings, not errors.
pub fn evaluate(
    policy: &Policy,
    policy_source: &str,
    patch: Patch,
    patch_source: &str,
) -> Result<Report, CageError> {
    let allowed = compile_globs(&policy.allowed_paths)?;
    let denied = compile_globs(&policy.denied_paths)?;
    let mut findings = Vec::new();

    push_limit(
        &mut findings,
        "file_budget_exceeded",
        patch.files.len(),
        policy.max_files,
        "files",
    );
    push_limit(
        &mut findings,
        "added_line_budget_exceeded",
        patch.added_lines,
        policy.max_added_lines,
        "added lines",
    );
    push_limit(
        &mut findings,
        "removed_line_budget_exceeded",
        patch.removed_lines,
        policy.max_removed_lines,
        "removed lines",
    );

    for file in &patch.files {
        let paths: BTreeSet<_> = [&file.old_path, &file.path].into_iter().collect();
        for path in paths {
            if !is_safe_path(path) {
                findings.push(path_finding(
                    "unsafe_path",
                    path,
                    "path is absolute, targets .git, or contains traversal components",
                ));
                continue;
            }
            if !allowed.is_match(path) {
                findings.push(path_finding(
                    "path_outside_allowlist",
                    path,
                    "path does not match any allowed_paths pattern",
                ));
            }
            if denied.is_match(path) {
                findings.push(path_finding(
                    "path_denied",
                    path,
                    "path matches a denied_paths pattern",
                ));
            }
        }

        match file.kind {
            ChangeKind::Added if !policy.allow_new_files => findings.push(path_finding(
                "new_file_denied",
                &file.path,
                "policy does not permit new files",
            )),
            ChangeKind::Deleted if !policy.allow_deleted_files => findings.push(path_finding(
                "deleted_file_denied",
                &file.path,
                "policy does not permit deleted files",
            )),
            ChangeKind::Renamed if !policy.allow_renames => findings.push(path_finding(
                "rename_denied",
                &file.path,
                "policy does not permit renames",
            )),
            _ => {}
        }
        if file.binary && !policy.allow_binary {
            findings.push(path_finding(
                "binary_patch_denied",
                &file.path,
                "policy does not permit binary patches",
            ));
        }
        if file.mode_changed && !policy.allow_mode_changes {
            findings.push(path_finding(
                "mode_change_denied",
                &file.path,
                "policy does not permit file mode changes",
            ));
        }
    }

    findings.sort_by(|left, right| {
        (&left.code, &left.path, &left.message).cmp(&(&right.code, &right.path, &right.message))
    });
    let passed = findings.is_empty();
    Ok(Report {
        schema_version: 1,
        passed,
        policy_sha256: digest(policy_source),
        patch_sha256: digest(patch_source),
        summary: Summary {
            files: patch.files.len(),
            added_lines: patch.added_lines,
            removed_lines: patch.removed_lines,
        },
        files: patch.files,
        findings,
    })
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet, CageError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|error| CageError::InvalidGlob {
            pattern: pattern.clone(),
            message: error.to_string(),
        })?;
        builder.add(glob);
    }
    builder.build().map_err(|error| CageError::InvalidGlob {
        pattern: "<glob set>".to_owned(),
        message: error.to_string(),
    })
}

fn push_limit(findings: &mut Vec<Finding>, code: &str, actual: usize, maximum: usize, unit: &str) {
    if actual > maximum {
        findings.push(Finding {
            code: code.to_owned(),
            path: None,
            message: format!("patch contains {actual} {unit}; policy allows at most {maximum}"),
        });
    }
}

fn path_finding(code: &str, path: &str, message: &str) -> Finding {
    Finding {
        code: code.to_owned(),
        path: Some(path.to_owned()),
        message: message.to_owned(),
    }
}

fn is_safe_path(value: &str) -> bool {
    let path = Path::new(value);
    if path.is_absolute() {
        return false;
    }
    let mut components = path.components();
    if matches!(components.next(), Some(Component::Normal(first)) if first == ".git") {
        return false;
    }
    !path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn digest(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: &str = r#"
version = 1
max_files = 2
max_added_lines = 3
max_removed_lines = 2
allowed_paths = ["src/**", "tests/**"]
denied_paths = ["src/generated/**"]
allow_new_files = true
"#;

    const SAFE_PATCH: &str = r"diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,3 @@
 old
-gone
+new
+extra
";

    #[test]
    fn parses_line_totals() {
        let patch = parse_patch(SAFE_PATCH).expect("patch should parse");
        assert_eq!(patch.added_lines, 2);
        assert_eq!(patch.removed_lines, 1);
        assert_eq!(patch.files[0].kind, ChangeKind::Modified);
    }

    #[test]
    fn safe_patch_passes_with_deterministic_hashes() {
        let policy = Policy::from_toml(POLICY).expect("policy should parse");
        let patch = parse_patch(SAFE_PATCH).expect("patch should parse");
        let report = evaluate(&policy, POLICY, patch, SAFE_PATCH).expect("evaluation should work");
        assert!(report.passed);
        assert_eq!(report.patch_sha256.len(), 64);
        assert_eq!(report.findings, Vec::new());
    }

    #[test]
    fn budgets_and_denied_paths_are_reported() {
        let policy = Policy::from_toml(POLICY).expect("policy should parse");
        let input = SAFE_PATCH.replace("src/lib.rs", "src/generated/lib.rs");
        let mut patch = parse_patch(&input).expect("patch should parse");
        patch.added_lines = 10;
        let report = evaluate(&policy, POLICY, patch, &input).expect("evaluation should work");
        let codes: Vec<_> = report
            .findings
            .iter()
            .map(|item| item.code.as_str())
            .collect();
        assert!(!report.passed);
        assert!(codes.contains(&"added_line_budget_exceeded"));
        assert!(codes.contains(&"path_denied"));
    }

    #[test]
    fn path_traversal_is_never_admitted() {
        let policy = Policy::from_toml(POLICY).expect("policy should parse");
        let input = SAFE_PATCH.replace("src/lib.rs", "src/../Cargo.toml");
        let patch = parse_patch(&input).expect("patch should parse");
        let report = evaluate(&policy, POLICY, patch, &input).expect("evaluation should work");
        assert!(report
            .findings
            .iter()
            .any(|item| item.code == "unsafe_path"));
    }

    #[test]
    fn new_binary_and_mode_changes_are_independent_findings() {
        let input = "diff --git a/src/blob.bin b/src/blob.bin\nnew file mode 100755\nold mode 100644\nnew mode 100755\nGIT binary patch\n";
        let policy = Policy::from_toml(POLICY).expect("policy should parse");
        let patch = parse_patch(input).expect("patch should parse");
        let report = evaluate(&policy, POLICY, patch, input).expect("evaluation should work");
        let codes: Vec<_> = report
            .findings
            .iter()
            .map(|item| item.code.as_str())
            .collect();
        assert!(codes.contains(&"binary_patch_denied"));
        assert!(codes.contains(&"mode_change_denied"));
        assert!(!codes.contains(&"new_file_denied"));
    }

    #[test]
    fn unsupported_or_empty_input_fails_closed() {
        assert!(matches!(parse_patch(""), Err(CageError::EmptyPatch)));
        assert!(matches!(
            parse_patch("diff --git \"a/a b\" \"b/a b\"\n"),
            Err(CageError::MalformedPatch { .. })
        ));
    }

    #[test]
    fn policy_rejects_unknown_fields_and_bad_version() {
        assert!(Policy::from_toml(&format!("{POLICY}\nunknown = true")).is_err());
        assert!(matches!(
            Policy::from_toml(&POLICY.replace("version = 1", "version = 2")),
            Err(CageError::PolicyVersion(2))
        ));
    }
}
