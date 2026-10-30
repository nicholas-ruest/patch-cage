# Patch Cage

`patch-cage` is a fail-closed Rust CLI for checking a Git patch against a small,
reviewable change envelope before a remediation worker or human applies it.

It answers a narrow question: **does this patch stay inside the paths, churn
budget, and file operations that were actually authorized?** It never applies
the patch or executes code.

## What works

- Parses conventional `git diff` file records and line hunks.
- Enforces allowed and denied path globs.
- Enforces file, added-line, and removed-line budgets.
- Independently controls additions, deletions, renames, binaries, and mode changes.
- Rejects absolute paths, `.git` targets, traversal components, ambiguous headers,
  unknown policy keys, and unsupported policy versions.
- Emits deterministic JSON receipts with policy and patch SHA-256 digests.
- Uses exit `0` for admission, `2` for policy denial, and `1` for malformed input
  or operational errors.

## Install

```console
cargo install --git https://github.com/nicholas-ruest/patch-cage
```

Or build the checkout with stable Rust 1.85 or newer:

```console
cargo build --release
```

## Use

Start from [`examples/patch-cage.toml`](examples/patch-cage.toml):

```toml
version = 1
max_files = 4
max_added_lines = 80
max_removed_lines = 40
allowed_paths = ["src/**", "tests/**"]
denied_paths = ["src/generated/**", "**/*.pem", "**/*.key"]
allow_new_files = true
allow_deleted_files = false
allow_renames = false
allow_binary = false
allow_mode_changes = false
```

Check a saved patch:

```console
patch-cage check --policy examples/patch-cage.toml --patch examples/safe.diff
```

Or gate the current working-tree diff and retain a JSON receipt:

```console
git diff | patch-cage check --policy patch-cage.toml --patch - --json > receipt.json
```

Example denial codes include `path_outside_allowlist`, `path_denied`,
`added_line_budget_exceeded`, `rename_denied`, `binary_patch_denied`, and
`mode_change_denied`.

## Verification

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo +1.85.0 check --all-features
```

CI repeats the three required stable checks and the Rust 1.85 MSRV check.

## Security boundary and limitations

`patch-cage` is a pre-application structural gate, not a sandbox, malware
detector, code-review substitute, or proof that a patch is correct. Run tests,
static analysis, and the changed program in an isolated environment after this
gate. The parser intentionally accepts conventional unquoted `diff --git`
headers only; Git patches with quoted paths (commonly paths containing spaces or
special characters), combined diffs, email patch bodies, or non-Git formats are
rejected rather than guessed. Content-pattern rules and signature verification
are not included in v0.1.0.

The idea was informed by the 2026-10-29 Enterprise Open-Source Radar signal
around security findings and isolated remediation workers. The implementation
is independent and uses no code from those projects.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
