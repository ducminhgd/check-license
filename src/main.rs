mod application;
mod domain;
mod infrastructure;
mod presentation;

use anyhow::{Context, Result};
use clap::Parser;

use application::ports::{
    ActivationChecker, AppScanner, KnowledgeBase, OnlineLicenseProvider, PackageLicenseProvider,
};
use application::use_cases::scan_and_check::ScanAndCheckUseCase;
use infrastructure::knowledge_base::bundled::BundledKnowledgeBase;
use infrastructure::online::NullOnlineLicenseProvider;
use presentation::table::TableRenderer;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
compile_error!(
    "Unsupported operating system. Only macOS, Linux, and Windows are currently supported."
);

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

    /// Query the internet (App Store / Flathub / Snapcraft) for apps not resolved offline
    #[arg(long)]
    online: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // KB is optional: if the file is missing or empty the tool falls back to package
    // manager and online lookup. We still try to load it for crack-indicator data.
    let kb = BundledKnowledgeBase::load().unwrap_or_else(|_| BundledKnowledgeBase::empty());
    dispatch(cli, &kb)
}

#[cfg(target_os = "macos")]
fn dispatch(cli: Cli, kb: &dyn KnowledgeBase) -> Result<()> {
    use infrastructure::online::macos::MacOsOnlineLicenseProvider;
    use infrastructure::package_manager::homebrew::HomebrewLicenseProvider;
    use infrastructure::platform::macos::{
        activation::MacOsActivationChecker, scanner::MacOsAppScanner,
    };

    let pkg = HomebrewLicenseProvider::load();
    if cli.online {
        run(cli, kb, &MacOsAppScanner, &MacOsActivationChecker, &pkg, &MacOsOnlineLicenseProvider)
    } else {
        run(cli, kb, &MacOsAppScanner, &MacOsActivationChecker, &pkg, &NullOnlineLicenseProvider)
    }
}

#[cfg(target_os = "linux")]
fn dispatch(cli: Cli, kb: &dyn KnowledgeBase) -> Result<()> {
    use infrastructure::online::linux::LinuxOnlineLicenseProvider;
    use infrastructure::package_manager::linux::LinuxLicenseProvider;
    use infrastructure::platform::linux::{
        activation::LinuxActivationChecker, scanner::LinuxAppScanner,
    };

    let pkg = LinuxLicenseProvider::load();
    if cli.online {
        run(cli, kb, &LinuxAppScanner, &LinuxActivationChecker, &pkg, &LinuxOnlineLicenseProvider)
    } else {
        run(cli, kb, &LinuxAppScanner, &LinuxActivationChecker, &pkg, &NullOnlineLicenseProvider)
    }
}

#[cfg(target_os = "windows")]
fn dispatch(cli: Cli, kb: &dyn KnowledgeBase) -> Result<()> {
    use infrastructure::online::windows::WindowsOnlineLicenseProvider;
    use infrastructure::package_manager::winget::WingetLicenseProvider;
    use infrastructure::platform::windows::{
        activation::WindowsActivationChecker, scanner::WindowsAppScanner,
    };

    if cli.online {
        run(
            cli,
            kb,
            &WindowsAppScanner,
            &WindowsActivationChecker,
            &WingetLicenseProvider,
            &WindowsOnlineLicenseProvider,
        )
    } else {
        run(
            cli,
            kb,
            &WindowsAppScanner,
            &WindowsActivationChecker,
            &WingetLicenseProvider,
            &NullOnlineLicenseProvider,
        )
    }
}

fn run(
    cli: Cli,
    kb: &dyn KnowledgeBase,
    scanner: &dyn AppScanner,
    checker: &dyn ActivationChecker,
    pkg: &dyn PackageLicenseProvider,
    online: &dyn OnlineLicenseProvider,
) -> Result<()> {
    let use_case = ScanAndCheckUseCase::new(scanner, kb, checker, pkg, online);
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
