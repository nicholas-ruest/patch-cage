use clap::{Parser, Subcommand};
use patch_cage::{evaluate, parse_patch, Policy, Report};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Check Git patches against a fail-closed change envelope"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Evaluate a patch without applying it.
    Check {
        /// TOML policy file.
        #[arg(short, long)]
        policy: PathBuf,
        /// Git patch file, or `-` to read standard input.
        #[arg(short = 'd', long)]
        patch: String,
        /// Emit a machine-readable JSON receipt.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("patch-cage: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check {
            policy,
            patch,
            json,
        } => {
            let policy_source = fs::read_to_string(&policy)?;
            let patch_source = read_patch(&patch)?;
            let policy = Policy::from_toml(&policy_source)?;
            let patch = parse_patch(&patch_source)?;
            let report = evaluate(&policy, &policy_source, patch, &patch_source)?;
            print_report(&report, json)?;
            Ok(if report.passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            })
        }
    }
}

fn read_patch(path: &str) -> io::Result<String> {
    if path == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    } else {
        fs::read_to_string(Path::new(path))
    }
}

fn print_report(report: &Report, json: bool) -> Result<(), serde_json::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else if report.passed {
        println!(
            "PASS: {} files, +{} -{} lines; patch sha256 {}",
            report.summary.files,
            report.summary.added_lines,
            report.summary.removed_lines,
            report.patch_sha256
        );
    } else {
        println!(
            "DENY: {} finding(s), {} files, +{} -{} lines",
            report.findings.len(),
            report.summary.files,
            report.summary.added_lines,
            report.summary.removed_lines
        );
        for finding in &report.findings {
            let path = finding.path.as_deref().unwrap_or("patch");
            println!("- {} [{}]: {}", finding.code, path, finding.message);
        }
    }
    Ok(())
}
