use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{Duration, Instant};

mod collatz_1000;
mod control_system;
mod deep_taxonomy_100000;
mod delfour;
mod euler_identity;
mod fibonacci;
mod goldbach_1000;
mod gps;
mod kaprekar_6174;
mod matrix_mechanics;
mod path_discovery;
mod polynomial;
mod report;
mod sudoku;

use report::CaseReport;

const CASE_NAMES: [&str; 13] = [
    "collatz-1000",
    "control-system",
    "deep-taxonomy-100000",
    "delfour",
    "euler-identity",
    "fibonacci",
    "goldbach-1000",
    "gps",
    "kaprekar-6174",
    "matrix-mechanics",
    "path-discovery",
    "polynomial",
    "sudoku",
];

const INDEX_SNAPSHOTS: [(&str, &str); 3] = [
    ("text", "list.txt"),
    ("text", "all.txt"),
    ("json", "all.json"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseName {
    Collatz1000,
    ControlSystem,
    DeepTaxonomy100000,
    Delfour,
    EulerIdentity,
    Fibonacci,
    Goldbach1000,
    Gps,
    Kaprekar6174,
    MatrixMechanics,
    PathDiscovery,
    Polynomial,
    Sudoku,
}

impl CaseName {
    fn as_str(self) -> &'static str {
        match self {
            CaseName::Collatz1000 => "collatz-1000",
            CaseName::ControlSystem => "control-system",
            CaseName::DeepTaxonomy100000 => "deep-taxonomy-100000",
            CaseName::Delfour => "delfour",
            CaseName::EulerIdentity => "euler-identity",
            CaseName::Fibonacci => "fibonacci",
            CaseName::Goldbach1000 => "goldbach-1000",
            CaseName::Gps => "gps",
            CaseName::Kaprekar6174 => "kaprekar-6174",
            CaseName::MatrixMechanics => "matrix-mechanics",
            CaseName::PathDiscovery => "path-discovery",
            CaseName::Polynomial => "polynomial",
            CaseName::Sudoku => "sudoku",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedCli {
    Run(RunOptions),
    Driver(DriverCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunOptions {
    command: RunCommand,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunCommand {
    List,
    All,
    Case(CaseName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DriverCommand {
    Check,
    Refresh,
    Show { case_name: CaseName, format: OutputFormat },
    Help,
}

fn other_error(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.into())
}

fn usage() -> &'static str {
    r#"Usage:
  arcproof [CASE | --all | --list] [--format text|json]
  arcproof show CASE [text|json]
  arcproof refresh
  arcproof check
  arcproof help

Case commands:
  --list, list     List the available cases.
  --all, all       Run every case.
  CASE             Run a single case such as 'sudoku' or 'collatz-1000'.

Snapshot commands (run from the repository root):
  refresh          Regenerate snapshots under snapshots/text and snapshots/json.
  check            Compare fresh output against the checked-in snapshots.
  show CASE        Print one case directly. Add 'json' for structured output.

Formatting:
  --format text    Emit human-readable ARC output.
  --format json    Emit structured JSON output.
  --text           Shorthand for '--format text'.
  --json           Shorthand for '--format json'.
"#
}

fn parse_output_format(value: &str) -> io::Result<OutputFormat> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown format '{other}'. use 'text' or 'json'"),
        )),
    }
}

fn parse_case_name(raw: &str) -> io::Result<CaseName> {
    match raw {
        "collatz-1000" | "collatz_1000" | "collatz-10000" | "collatz_10000" | "collatz" => {
            Ok(CaseName::Collatz1000)
        }
        "control-system" | "control_system" => Ok(CaseName::ControlSystem),
        "deep-taxonomy-100000" | "deep_taxonomy_100000" => Ok(CaseName::DeepTaxonomy100000),
        "delfour" => Ok(CaseName::Delfour),
        "euler-identity" | "euler_identity" => Ok(CaseName::EulerIdentity),
        "fibonacci" => Ok(CaseName::Fibonacci),
        "goldbach-1000" | "goldbach_1000" | "goldbach" => Ok(CaseName::Goldbach1000),
        "gps" => Ok(CaseName::Gps),
        "kaprekar-6174" | "kaprekar_6174" => Ok(CaseName::Kaprekar6174),
        "matrix-mechanics" | "matrix_mechanics" | "matrix" => Ok(CaseName::MatrixMechanics),
        "path-discovery" | "path_discovery" => Ok(CaseName::PathDiscovery),
        "polynomial" => Ok(CaseName::Polynomial),
        "sudoku" => Ok(CaseName::Sudoku),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "unknown case '{other}'. available cases:\n  {}\n\nextra commands:\n  --list\n  --all\n  show CASE [text|json]\n  refresh\n  check\n\nextra options:\n  --format text\n  --format json",
                CASE_NAMES.join("\n  ")
            ),
        )),
    }
}

fn parse_run_args(args: impl IntoIterator<Item = String>) -> io::Result<RunOptions> {
    let mut format = OutputFormat::Text;
    let mut command = RunCommand::All;
    let mut saw_command = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--format" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing value after --format")
                })?;
                format = parse_output_format(&value)?;
            }
            "--json" => format = OutputFormat::Json,
            "--text" => format = OutputFormat::Text,
            "--list" | "list" => {
                if saw_command {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "multiple commands provided; use one case name, --all, or --list",
                    ));
                }
                command = RunCommand::List;
                saw_command = true;
            }
            "--all" | "all" => {
                if saw_command {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "multiple commands provided; use one case name, --all, or --list",
                    ));
                }
                command = RunCommand::All;
                saw_command = true;
            }
            raw_case => {
                if saw_command {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "multiple commands provided; use one case name, --all, or --list",
                    ));
                }
                command = RunCommand::Case(parse_case_name(raw_case)?);
                saw_command = true;
            }
        }
    }

    Ok(RunOptions { command, format })
}

fn parse_show_args(args: &[String]) -> io::Result<DriverCommand> {
    let (raw_case, rest) = args.split_first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("missing case name for 'show'\n\n{}", usage()),
        )
    })?;

    let case_name = parse_case_name(raw_case)?;
    let mut format = OutputFormat::Text;

    let mut iter = rest.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "text" | "json" => format = parse_output_format(arg)?,
            "--format" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing value after --format")
                })?;
                format = parse_output_format(value)?;
            }
            "--json" => format = OutputFormat::Json,
            "--text" => format = OutputFormat::Text,
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument for 'show': {other}"),
                ));
            }
        }
    }

    Ok(DriverCommand::Show { case_name, format })
}

fn parse_cli(args: impl IntoIterator<Item = String>) -> io::Result<ParsedCli> {
    let args: Vec<String> = args.into_iter().collect();

    match args.first().map(String::as_str) {
        Some("check") => {
            if args.len() > 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "'check' does not take additional arguments",
                ));
            }
            Ok(ParsedCli::Driver(DriverCommand::Check))
        }
        Some("refresh") => {
            if args.len() > 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "'refresh' does not take additional arguments",
                ));
            }
            Ok(ParsedCli::Driver(DriverCommand::Refresh))
        }
        Some("show") => Ok(ParsedCli::Driver(parse_show_args(&args[1..])?)),
        Some("help") | Some("-h") | Some("--help") => Ok(ParsedCli::Driver(DriverCommand::Help)),
        _ => Ok(ParsedCli::Run(parse_run_args(args.into_iter())?)),
    }
}

fn case_names() -> &'static [&'static str] {
    &CASE_NAMES
}

fn print_case_list() {
    for case in case_names() {
        println!("{case}");
    }
}

fn case_report(case_name: CaseName) -> io::Result<CaseReport> {
    match case_name {
        CaseName::Collatz1000 => collatz_1000::report(),
        CaseName::ControlSystem => control_system::report(),
        CaseName::DeepTaxonomy100000 => deep_taxonomy_100000::report(),
        CaseName::Delfour => delfour::report(),
        CaseName::EulerIdentity => euler_identity::report(),
        CaseName::Fibonacci => fibonacci::report(),
        CaseName::Goldbach1000 => goldbach_1000::report(),
        CaseName::Gps => gps::report(),
        CaseName::Kaprekar6174 => kaprekar_6174::report(),
        CaseName::MatrixMechanics => matrix_mechanics::report(),
        CaseName::PathDiscovery => path_discovery::report(),
        CaseName::Polynomial => polynomial::report(),
        CaseName::Sudoku => sudoku::report(),
    }
}

fn emit_json_report(report: &CaseReport, mut writer: impl Write) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut writer, report)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
    writer.write_all(b"\n")
}


const TIMING_COLOR: &str = "\x1b[38;5;45m";
const TIMING_ERROR_COLOR: &str = "\x1b[38;5;203m";
const ANSI_RESET: &str = "\x1b[0m";

fn format_elapsed(duration: Duration) -> String {
    format!("{:.3} ms", duration.as_secs_f64() * 1_000.0)
}

fn print_timing(label: &str, duration: Duration) {
    eprintln!(
        "{TIMING_COLOR}[timing]{ANSI_RESET} {label}: {}",
        format_elapsed(duration)
    );
}

fn print_timing_failure(label: &str, duration: Duration, error: &io::Error) {
    eprintln!(
        "{TIMING_ERROR_COLOR}[timing]{ANSI_RESET} {label} failed after {}: {error}",
        format_elapsed(duration)
    );
}

fn with_case_timing<T>(case_name: CaseName, action: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    let start = Instant::now();
    let result = action();
    let duration = start.elapsed();
    let label = format!("case '{}'", case_name.as_str());

    match &result {
        Ok(_) => print_timing(&label, duration),
        Err(error) => print_timing_failure(&label, duration, error),
    }

    result
}

fn run_case_text(case_name: CaseName) -> io::Result<()> {
    match case_name {
        CaseName::Collatz1000 => collatz_1000::run_and_print(),
        CaseName::ControlSystem => control_system::run_and_print(),
        CaseName::DeepTaxonomy100000 => deep_taxonomy_100000::run_and_print(),
        CaseName::Delfour => delfour::run_and_print(),
        CaseName::EulerIdentity => euler_identity::run_and_print(),
        CaseName::Fibonacci => fibonacci::run_and_print(),
        CaseName::Goldbach1000 => goldbach_1000::run_and_print(),
        CaseName::Gps => gps::run_and_print(),
        CaseName::Kaprekar6174 => kaprekar_6174::run_and_print(),
        CaseName::MatrixMechanics => matrix_mechanics::run_and_print(),
        CaseName::PathDiscovery => path_discovery::run_and_print(),
        CaseName::Polynomial => polynomial::run_and_print(),
        CaseName::Sudoku => sudoku::run_and_print(),
    }
}

fn all_case_names() -> [CaseName; 13] {
    [
        CaseName::Collatz1000,
        CaseName::ControlSystem,
        CaseName::DeepTaxonomy100000,
        CaseName::Delfour,
        CaseName::EulerIdentity,
        CaseName::Fibonacci,
        CaseName::Goldbach1000,
        CaseName::Gps,
        CaseName::Kaprekar6174,
        CaseName::MatrixMechanics,
        CaseName::PathDiscovery,
        CaseName::Polynomial,
        CaseName::Sudoku,
    ]
}

fn run_all_cases(format: OutputFormat) -> io::Result<()> {
    match format {
        OutputFormat::Text => {
            let runners: [(CaseName, fn() -> io::Result<()>); 13] = [
                (CaseName::Collatz1000, collatz_1000::run_and_print),
                (CaseName::ControlSystem, control_system::run_and_print),
                (CaseName::DeepTaxonomy100000, deep_taxonomy_100000::run_and_print),
                (CaseName::Delfour, delfour::run_and_print),
                (CaseName::EulerIdentity, euler_identity::run_and_print),
                (CaseName::Fibonacci, fibonacci::run_and_print),
                (CaseName::Goldbach1000, goldbach_1000::run_and_print),
                (CaseName::Gps, gps::run_and_print),
                (CaseName::Kaprekar6174, kaprekar_6174::run_and_print),
                (CaseName::MatrixMechanics, matrix_mechanics::run_and_print),
                (CaseName::PathDiscovery, path_discovery::run_and_print),
                (CaseName::Polynomial, polynomial::run_and_print),
                (CaseName::Sudoku, sudoku::run_and_print),
            ];

            for (index, (case_name, run)) in runners.into_iter().enumerate() {
                if index > 0 {
                    println!();
                    println!("------------------------------------------------------------------------");
                    println!();
                }
                with_case_timing(case_name, run)?;
            }
            Ok(())
        }
        OutputFormat::Json => {
            let mut reports = Vec::with_capacity(all_case_names().len());
            for case_name in all_case_names() {
                reports.push(with_case_timing(case_name, || case_report(case_name))?);
            }
            let mut stdout = io::stdout();
            serde_json::to_writer_pretty(&mut stdout, &reports)
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            stdout.write_all(b"\n")
        }
    }
}

fn run_single_case(case_name: CaseName, format: OutputFormat) -> io::Result<()> {
    with_case_timing(case_name, || match format {
        OutputFormat::Text => run_case_text(case_name),
        OutputFormat::Json => {
            let report = case_report(case_name)?;
            emit_json_report(&report, io::stdout())
        }
    })
}

fn repo_root() -> io::Result<PathBuf> {
    env::current_dir()
}

fn snapshot_root(root: &Path) -> PathBuf {
    root.join("snapshots")
}

fn temp_snapshot_root(root: &Path) -> PathBuf {
    root.join("target").join("arcproof-pilot")
}

fn expected_snapshot_paths(root: &Path) -> Vec<PathBuf> {
    let snapshot_root = snapshot_root(root);
    let mut paths = Vec::with_capacity(INDEX_SNAPSHOTS.len() + (CASE_NAMES.len() * 2));

    for (kind, file_name) in INDEX_SNAPSHOTS {
        paths.push(snapshot_root.join(kind).join(file_name));
    }

    for case in CASE_NAMES {
        paths.push(snapshot_root.join("text").join(format!("{case}.txt")));
        paths.push(snapshot_root.join("json").join(format!("{case}.json")));
    }

    paths
}

fn discovered_snapshot_paths(root: &Path) -> io::Result<Vec<PathBuf>> {
    let snapshots = snapshot_root(root);
    let mut paths = Vec::new();

    for kind in ["text", "json"] {
        let dir = snapshots.join(kind);
        if !dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.file_name() != Some(OsStr::new(".gitkeep")) {
                paths.push(path);
            }
        }
    }

    paths.sort();
    Ok(paths)
}

fn run_self_capture<I, S>(args: I) -> io::Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let executable = env::current_exe()?;
    let output = ProcessCommand::new(&executable).args(args).output()?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(other_error(format!(
            "failed to run '{}' for snapshot generation: {}",
            executable.display(),
            stderr.trim()
        )))
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}


fn capture_snapshot(label: &str, args: &[&str]) -> io::Result<Vec<u8>> {
    let start = Instant::now();
    let result = run_self_capture(args.iter().copied());
    let duration = start.elapsed();

    match &result {
        Ok(_) => print_timing(label, duration),
        Err(error) => print_timing_failure(&label, duration, error),
    }

    result
}

fn write_snapshots(out_root: &Path) -> io::Result<()> {
    write_bytes(
        &out_root.join("text/list.txt"),
        &capture_snapshot("snapshot list", &["--list"])?
    )?;
    write_bytes(
        &out_root.join("text/all.txt"),
        &capture_snapshot("snapshot all text", &["--all"])?
    )?;
    write_bytes(
        &out_root.join("json/all.json"),
        &capture_snapshot("snapshot all json", &["--all", "--format", "json"])?
    )?;

    for case in all_case_names() {
        let case_name = case.as_str();
        let text_label = format!("snapshot case '{case_name}' text");
        write_bytes(
            &out_root.join("text").join(format!("{case_name}.txt")),
            &capture_snapshot(&text_label, &[case_name])?,
        )?;

        let json_label = format!("snapshot case '{case_name}' json");
        write_bytes(
            &out_root.join("json").join(format!("{case_name}.json")),
            &capture_snapshot(&json_label, &[case_name, "--format", "json"])?
        )?;
    }

    Ok(())
}

fn require_snapshots(root: &Path) -> io::Result<()> {
    let expected = expected_snapshot_paths(root);
    let discovered = discovered_snapshot_paths(root)?;
    let mut missing = Vec::new();
    let mut unexpected = Vec::new();

    for path in &expected {
        if !path.is_file() {
            missing.push(path.clone());
        }
    }

    for path in discovered {
        if !expected.contains(&path) {
            unexpected.push(path);
        }
    }

    if missing.is_empty() && unexpected.is_empty() {
        return Ok(());
    }

    for path in &missing {
        eprintln!("missing snapshot: {}", path.display());
    }
    for path in &unexpected {
        eprintln!("unexpected snapshot: {}", path.display());
    }
    eprintln!();
    eprintln!("Run 'cargo run --release -- refresh' from the repository root to generate the snapshots.");

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "{} snapshot file(s) are missing and {} are unexpected",
            missing.len(),
            unexpected.len()
        ),
    ))
}

fn diff_summary(expected: &Path, actual: &Path, label: &Path) -> io::Result<()> {
    let expected_bytes = fs::read(expected)?;
    let actual_bytes = fs::read(actual)?;

    if expected_bytes == actual_bytes {
        return Ok(());
    }

    let expected_text = String::from_utf8_lossy(&expected_bytes);
    let actual_text = String::from_utf8_lossy(&actual_bytes);
    let expected_lines: Vec<_> = expected_text.lines().collect();
    let actual_lines: Vec<_> = actual_text.lines().collect();
    let max_len = expected_lines.len().max(actual_lines.len());

    let mut first_difference = None;
    for index in 0..max_len {
        let expected_line = expected_lines.get(index).copied();
        let actual_line = actual_lines.get(index).copied();
        if expected_line != actual_line {
            first_difference = Some((index + 1, expected_line, actual_line));
            break;
        }
    }

    eprintln!("snapshot differs: {}", label.display());
    match first_difference {
        Some((line, expected_line, actual_line)) => {
            eprintln!("  first difference at line {line}");
            eprintln!("  expected: {}", expected_line.unwrap_or("<missing line>"));
            eprintln!("  actual  : {}", actual_line.unwrap_or("<missing line>"));
        }
        None => {
            eprintln!(
                "  content differs in line endings or trailing whitespace (expected {} bytes, actual {} bytes)",
                expected_bytes.len(),
                actual_bytes.len()
            );
        }
    }

    Ok(())
}

fn check_snapshots() -> io::Result<()> {
    let root = repo_root()?;
    require_snapshots(&root)?;

    let temp_root = temp_snapshot_root(&root);
    if temp_root.exists() {
        fs::remove_dir_all(&temp_root)?;
    }
    fs::create_dir_all(&temp_root)?;
    write_snapshots(&temp_root)?;

    let snapshots = snapshot_root(&root);
    let mut failed = false;

    for expected in expected_snapshot_paths(&root) {
        let relative = expected
            .strip_prefix(&snapshots)
            .expect("snapshot path should be under the snapshot root");
        let actual = temp_root.join(relative);

        if !actual.is_file() {
            eprintln!("missing generated snapshot: {}", actual.display());
            failed = true;
            continue;
        }

        let expected_bytes = fs::read(&expected)?;
        let actual_bytes = fs::read(&actual)?;
        if expected_bytes != actual_bytes {
            diff_summary(&expected, &actual, relative)?;
            failed = true;
        }
    }

    if failed {
        eprintln!();
        eprintln!("Snapshot drift detected. Review the differences, then run 'cargo run --release -- refresh' if the change is intentional.");
        return Err(other_error("snapshot drift detected"));
    }

    println!("All snapshots match.");
    Ok(())
}

fn refresh_snapshots() -> io::Result<()> {
    let root = repo_root()?;
    let snapshots = snapshot_root(&root);
    fs::create_dir_all(snapshots.join("text"))?;
    fs::create_dir_all(snapshots.join("json"))?;
    write_snapshots(&snapshots)?;
    println!("Snapshots refreshed under {}", snapshots.display());
    Ok(())
}

fn main() -> io::Result<()> {
    match parse_cli(env::args().skip(1))? {
        ParsedCli::Run(options) => match options.command {
            RunCommand::List => {
                print_case_list();
                Ok(())
            }
            RunCommand::All => run_all_cases(options.format),
            RunCommand::Case(case_name) => run_single_case(case_name, options.format),
        },
        ParsedCli::Driver(command) => match command {
            DriverCommand::Check => check_snapshots(),
            DriverCommand::Refresh => refresh_snapshots(),
            DriverCommand::Show { case_name, format } => run_single_case(case_name, format),
            DriverCommand::Help => {
                print!("{}", usage());
                Ok(())
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn accepts_hyphenated_and_snake_case_aliases() {
        assert!(matches!(parse_case_name("collatz-1000").unwrap(), CaseName::Collatz1000));
        assert!(matches!(parse_case_name("collatz_1000").unwrap(), CaseName::Collatz1000));
        assert!(matches!(parse_case_name("collatz-10000").unwrap(), CaseName::Collatz1000));
        assert!(matches!(parse_case_name("collatz_10000").unwrap(), CaseName::Collatz1000));
        assert!(matches!(parse_case_name("collatz").unwrap(), CaseName::Collatz1000));
        assert!(matches!(parse_case_name("control-system").unwrap(), CaseName::ControlSystem));
        assert!(matches!(parse_case_name("control_system").unwrap(), CaseName::ControlSystem));
        assert!(matches!(parse_case_name("deep-taxonomy-100000").unwrap(), CaseName::DeepTaxonomy100000));
        assert!(matches!(parse_case_name("deep_taxonomy_100000").unwrap(), CaseName::DeepTaxonomy100000));
        assert!(matches!(parse_case_name("euler-identity").unwrap(), CaseName::EulerIdentity));
        assert!(matches!(parse_case_name("euler_identity").unwrap(), CaseName::EulerIdentity));
        assert!(matches!(parse_case_name("goldbach-1000").unwrap(), CaseName::Goldbach1000));
        assert!(matches!(parse_case_name("goldbach_1000").unwrap(), CaseName::Goldbach1000));
        assert!(matches!(parse_case_name("goldbach").unwrap(), CaseName::Goldbach1000));
        assert!(matches!(parse_case_name("kaprekar-6174").unwrap(), CaseName::Kaprekar6174));
        assert!(matches!(parse_case_name("kaprekar_6174").unwrap(), CaseName::Kaprekar6174));
        assert!(matches!(parse_case_name("path-discovery").unwrap(), CaseName::PathDiscovery));
        assert!(matches!(parse_case_name("path_discovery").unwrap(), CaseName::PathDiscovery));
        assert!(matches!(parse_case_name("polynomial").unwrap(), CaseName::Polynomial));
        assert!(matches!(parse_case_name("sudoku").unwrap(), CaseName::Sudoku));
    }

    #[test]
    fn unknown_case_lists_available_options() {
        let error = parse_case_name("nope").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("unknown case 'nope'"));
        assert!(message.contains("collatz-1000"));
        assert!(message.contains("goldbach-1000"));
        assert!(message.contains("path-discovery"));
        assert!(message.contains("show CASE [text|json]"));
        assert!(message.contains("refresh"));
        assert!(message.contains("check"));
        assert!(message.contains("--format json"));
    }

    #[test]
    fn parses_format_before_or_after_command() {
        assert_eq!(
            parse_run_args(["--format".into(), "json".into(), "goldbach-1000".into()]).unwrap(),
            RunOptions {
                command: RunCommand::Case(CaseName::Goldbach1000),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_run_args(["goldbach-1000".into(), "--format".into(), "json".into()]).unwrap(),
            RunOptions {
                command: RunCommand::Case(CaseName::Goldbach1000),
                format: OutputFormat::Json,
            }
        );
    }

    #[test]
    fn parses_driver_commands() {
        assert_eq!(
            parse_cli(["check".into()]).unwrap(),
            ParsedCli::Driver(DriverCommand::Check)
        );
        assert_eq!(
            parse_cli(["refresh".into()]).unwrap(),
            ParsedCli::Driver(DriverCommand::Refresh)
        );
        assert_eq!(
            parse_cli(["show".into(), "sudoku".into(), "json".into()]).unwrap(),
            ParsedCli::Driver(DriverCommand::Show {
                case_name: CaseName::Sudoku,
                format: OutputFormat::Json,
            })
        );
    }

    #[test]
    fn expected_snapshot_list_covers_suite_and_indexes() {
        let root = Path::new(".");
        let expected = expected_snapshot_paths(root);
        let expected_set: BTreeSet<_> = expected.iter().cloned().collect();

        assert_eq!(expected.len(), expected_set.len(), "snapshot paths should be unique");
        assert!(expected.contains(&root.join("snapshots/text/list.txt")));
        assert!(expected.contains(&root.join("snapshots/text/all.txt")));
        assert!(expected.contains(&root.join("snapshots/json/all.json")));

        for case in CASE_NAMES {
            assert!(expected.contains(&root.join("snapshots/text").join(format!("{case}.txt"))));
            assert!(expected.contains(&root.join("snapshots/json").join(format!("{case}.json"))));
        }
    }

    #[test]
    fn formats_elapsed_times_in_milliseconds() {
        assert_eq!(format_elapsed(Duration::from_nanos(500)), "0.001 ms");
        assert_eq!(format_elapsed(Duration::from_nanos(499)), "0.000 ms");
        assert_eq!(format_elapsed(Duration::from_nanos(1_234)), "0.001 ms");
        assert_eq!(format_elapsed(Duration::from_nanos(12_345_678)), "12.346 ms");
        assert_eq!(format_elapsed(Duration::from_nanos(1_234_567_890)), "1234.568 ms");
    }
}
