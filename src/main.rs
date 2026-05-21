mod application;
mod domain;
mod infrastructure;
mod presentation;

use anyhow::{Context, Result};
use clap::Parser;

use application::{ports::KnowledgeBase, use_cases::scan_and_check::ScanAndCheckUseCase};
use infrastructure::knowledge_base::bundled::BundledKnowledgeBase;
use presentation::table::TableRenderer;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
compile_error!("Unsupported operating system. Only macOS, Linux, and Windows are currently supported.");

#[derive(Parser)]
#[command(
    name = "check-license",
    about = "Audit installed applications for license compliance and suspected crack software",
    version
)]
struct Cli {
    /// Also flag applications whose license does not permit commercial use
    #[arg(long)]
    work: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let kb = BundledKnowledgeBase::load().context("Failed to load knowledge base")?;
    dispatch(cli, &kb)
}

#[cfg(target_os = "macos")]
fn dispatch(cli: Cli, kb: &dyn application::ports::KnowledgeBase) -> Result<()> {
    use infrastructure::platform::macos::{
        activation::MacOsActivationChecker, scanner::MacOsAppScanner,
    };
    run(cli, kb, &MacOsAppScanner, &MacOsActivationChecker)
}

#[cfg(target_os = "linux")]
fn dispatch(cli: Cli, kb: &dyn application::ports::KnowledgeBase) -> Result<()> {
    use infrastructure::platform::linux::{
        activation::LinuxActivationChecker, scanner::LinuxAppScanner,
    };
    run(cli, kb, &LinuxAppScanner, &LinuxActivationChecker)
}

#[cfg(target_os = "windows")]
fn dispatch(cli: Cli, kb: &dyn application::ports::KnowledgeBase) -> Result<()> {
    use infrastructure::platform::windows::{
        activation::WindowsActivationChecker, scanner::WindowsAppScanner,
    };
    run(cli, kb, &WindowsAppScanner, &WindowsActivationChecker)
}

fn run(
    cli: Cli,
    kb: &dyn application::ports::KnowledgeBase,
    scanner: &dyn application::ports::AppScanner,
    checker: &dyn application::ports::ActivationChecker,
) -> Result<()> {
    let use_case = ScanAndCheckUseCase::new(scanner, kb, checker);
    let results = use_case.execute().context("Failed to scan applications")?;

    let renderer = TableRenderer { show_work_column: cli.work };
    println!("{}", renderer.render(&results));
    println!();
    println!("{}", renderer.render_summary(&results, cli.work));

    let has_cracks = results.iter().any(|r| r.crack_suspected);
    let has_work_violations = cli.work && results.iter().any(|r| !r.work_allowed);

    if has_cracks {
        std::process::exit(1);
    }
    if has_work_violations {
        std::process::exit(2);
    }

    Ok(())
}
