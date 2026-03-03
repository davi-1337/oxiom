mod cdp;
mod runner;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "oxiom", about = "Structure-aware Chrome CSS/layout fuzzer — runs forever until Ctrl+C")]
struct Args {
    /// Path to Chrome/Chromium ASAN executable.
    #[arg(long, default_value = "/usr/bin/chromium")]
    path: PathBuf,

    /// Max iterations (0 = infinite, default).
    #[arg(long, default_value_t = 0)]
    iterations: u64,

    /// Timeout per test case in milliseconds.
    #[arg(long, default_value_t = 8000)]
    timeout: u64,

    /// Parallel Chrome processes.
    #[arg(short, long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Output directory for generated test cases.
    #[arg(long, default_value = "./out/test")]
    output_dir: PathBuf,

    /// Directory to save crash reproducers.
    #[arg(long, default_value = "./out/crashes")]
    crash_dir: PathBuf,

    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Virtual time budget for Chrome in milliseconds.
    #[arg(long, default_value_t = 3000)]
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

    // Ctrl+C handler: 0=running, 1=graceful shutdown, 2=force exit
    let shutdown = Arc::new(AtomicU8::new(0));
    let shutdown_clone = shutdown.clone();

    ctrlc_handler(shutdown_clone);

    let config = runner::RunnerConfig {
        chrome_path: args.path,
        iterations: args.iterations, // 0 = infinite
        timeout_ms: args.timeout,
        output_dir: args.output_dir,
        crash_dir: args.crash_dir,
        jobs: args.jobs,
        verbose: args.verbose,
        virtual_time_budget: args.virtual_time_budget,
        seed: args.seed,
        buf_size: args.buf_size.clamp(2048, 16384),
        shutdown,
    };

    runner::run(config).await?;

    Ok(())
}

fn ctrlc_handler(shutdown: Arc<AtomicU8>) {
    let _ = ctrlc::set_handler(move || {
        let prev = shutdown.fetch_add(1, Ordering::SeqCst);
        match prev {
            0 => {
                eprintln!("\n[oxiom] Ctrl+C received — finishing in-flight tasks... (press Ctrl+C again to force exit)");
            }
            _ => {
                eprintln!("\n[oxiom] Force exit.");
                std::process::exit(1);
            }
        }
    });
}
