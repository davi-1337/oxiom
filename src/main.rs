mod cdp;
mod runner;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "oxiom", about = "Structure-aware Chrome CSS fuzzer")]
struct Args {
    /// Path to Chrome/Chromium executable.
    #[arg(long, default_value = "/usr/bin/chromium")]
    path: PathBuf,

    /// Number of fuzzing iterations.
    #[arg(long, default_value_t = 10000)]
    iterations: u64,

    /// Timeout per test case in milliseconds.
    #[arg(long, default_value_t = 5000)]
    timeout: u64,

    /// Parallel Chrome processes.
    #[arg(short, long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Output directory for generated test cases.
    #[arg(long, default_value = "./out/test")]
    output_dir: PathBuf,

    /// Directory to save crash reproducers.
    #[arg(long, default_value = "./out/crashs")]
    crash_dir: PathBuf,

    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Run continuously until Ctrl-C (ignore --iterations).
    #[arg(long)]
    continuous: bool,

    /// Virtual time budget for Chrome in milliseconds.
    #[arg(long, default_value_t = 2000)]
    virtual_time_budget: u64,

    /// RNG seed (0 = random).
    #[arg(long, default_value_t = 0)]
    seed: u64,

    /// Byte buffer size for Arbitrary generation.
    #[arg(long, default_value_t = 4096)]
    buf_size: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    if !args.path.exists() {
        anyhow::bail!(
            "Chrome not found at: {}. Use --path to specify.",
            args.path.display()
        );
    }

    let config = runner::RunnerConfig {
        chrome_path: args.path,
        iterations: args.iterations,
        timeout_ms: args.timeout,
        output_dir: args.output_dir,
        crash_dir: args.crash_dir,
        jobs: args.jobs,
        verbose: args.verbose,
        continuous: args.continuous,
        virtual_time_budget: args.virtual_time_budget,
        seed: args.seed,
        buf_size: args.buf_size.clamp(2048, 16384),
    };

    runner::run(config).await?;

    Ok(())
}
