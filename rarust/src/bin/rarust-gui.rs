//! `rarust-gui` — graphical archive browser (egui).

use std::process::ExitCode;

use clap::Parser;
use rarust::gui::run_gui;

/// Graphical RAR archive browser.
#[derive(Parser, Debug)]
#[command(name = "rarust-gui", version, about, long_about = None)]
struct GuiCli {
    /// Archive path to open (optional — opens file picker when omitted).
    archive: Option<String>,

    /// UI language (en, zh-CN, zh-TW). Overrides auto-detection.
    #[arg(short = 'l', long = "lang")]
    lang: Option<String>,

    /// Password for encrypted archives.
    #[arg(short = 'p', long = "password")]
    password: Option<String>,
}

fn main() -> ExitCode {
    let args = GuiCli::parse();
    let locale = args.lang.as_deref().and_then(|s| s.parse().ok());
    match run_gui(args.archive.as_deref(), locale, args.password) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(2)
        }
    }
}