//! FlowModus sidecar binary — thin process wrapper over the library CLI.
//!
//! Usage:
//!   flowmodus judge <text>    JP-1/JP-2 Rules verdicts (0 tokens)
//!   flowmodus measure <text>  STE (度) — standard token equivalent
//!   flowmodus verify <json>   canonical form + sha256 (检定)

use flowmodus::cli::Cli;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(Cli::run(&args));
}
