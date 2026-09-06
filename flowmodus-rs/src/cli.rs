//! CLI entrypoint (sidecar) — comes alive in R-3/R-4.
//!
//! Planned subcommands (naming may evolve; philosophy: accurate + short):
//! `serve` (sidecar proxy), `verify` (registry), `deviate` (deviation report).

/// CLI stub — real argument parsing lands in R-3.
pub struct Cli;

impl Cli {
    /// Stub: no-op.
    pub fn run_stub() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_runs_silently() {
        Cli::run_stub();
    }
}
