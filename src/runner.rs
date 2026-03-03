use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use arbitrary::{Arbitrary, Unstructured};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use tokio::sync::Semaphore;

use oxiom_generator::FuzzProgram;
use oxiom_serializer::serialize;

use crate::cdp::{self, CrashType, TestResult};

pub struct RunnerConfig {
    pub chrome_path: PathBuf,
    pub iterations: u64,       // 0 = infinite
    pub timeout_ms: u64,
    pub output_dir: PathBuf,
    pub crash_dir: PathBuf,
    pub jobs: usize,
    pub verbose: bool,
    pub virtual_time_budget: u64,
    pub seed: u64,
    pub buf_size: usize,
    pub shutdown: Arc<AtomicU8>, // 0=run, 1+=stop
}

/// Crash deduplicator — hash ASAN stack traces to prevent duplicates.
struct CrashDeduplicator {
    seen: HashSet<u64>,
}

impl CrashDeduplicator {
    fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    fn hash_crash(&self, log: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        let mut frame_count = 0;

        for line in log.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') && frame_count < 5 {
                if let Some(in_pos) = trimmed.find(" in ") {
                    let after_in = &trimmed[in_pos + 4..];
                    let func_name = after_in.split_whitespace().next().unwrap_or(after_in);
                    for b in func_name.bytes() {
                        h ^= b as u64;
                        h = h.wrapping_mul(0x100000001b3);
                    }
                    frame_count += 1;
                }
            }

            if trimmed.contains("ERROR: AddressSanitizer:")
                || trimmed.contains("heap-use-after-free")
                || trimmed.contains("heap-buffer-overflow")
                || trimmed.contains("SEGV on unknown address")
                || trimmed.contains("double-free")
                || trimmed.contains("use-after-poison")
                || trimmed.contains("stack-buffer-overflow")
                || trimmed.contains("alloc-dealloc-mismatch")
            {
                for b in trimmed.bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
        }

        h
    }

    fn is_new(&mut self, log: &str) -> bool {
        let hash = self.hash_crash(log);
        self.seen.insert(hash)
    }
}

fn build_crash_metadata(
    iteration: u64,
    crash_type: CrashType,
    signal: Option<i32>,
    exit_code: Option<i32>,
    html_size: usize,
    seed_info: &str,
    asan_error_type: Option<&str>,
) -> String {
    let signal_str = match signal {
        Some(s) => format!("{}", s),
        None => "null".to_string(),
    };
    let exit_code_str = match exit_code {
        Some(c) => format!("{}", c),
        None => "null".to_string(),
    };
    let asan_type_str = match asan_error_type {
        Some(t) => format!("\"{}\"", t),
        None => "null".to_string(),
    };

    format!(
        r#"{{
  "iteration": {},
  "crash_type": "{}",
  "asan_error_type": {},
  "signal": {},
  "exit_code": {},
  "html_size_bytes": {},
  "seed_info": "{}",
  "timestamp": "{}"
}}"#,
        iteration,
        crash_type.as_str(),
        asan_type_str,
        signal_str,
        exit_code_str,
        html_size,
        seed_info,
        unix_timestamp(),
    )
}

fn unix_timestamp() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", dur.as_secs())
}

/// Format duration as human-readable (e.g., "2h 15m 30s").
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

pub async fn run(config: RunnerConfig) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&config.output_dir).await?;
    tokio::fs::create_dir_all(&config.crash_dir).await?;

    let timeout = Duration::from_millis(config.timeout_ms);
    let jobs = config.jobs;
    let infinite = config.iterations == 0;

    // Determine actual seed
    let actual_seed = if config.seed != 0 {
        config.seed
    } else {
        rand::random::<u64>()
    };

    tracing::info!("=== oxiom fuzzer ===");
    tracing::info!(
        "mode: {} | jobs: {} | timeout: {}ms | vt-budget: {}ms | buf: {}B | seed: {}",
        if infinite { "INFINITE".to_string() } else { format!("{} iterations", config.iterations) },
        jobs,
        config.timeout_ms,
        config.virtual_time_budget,
        config.buf_size,
        actual_seed,
    );
    tracing::info!("chrome: {}", config.chrome_path.display());
    tracing::info!("crashes: {}", config.crash_dir.display());
    if infinite {
        tracing::info!("Press Ctrl+C to stop gracefully, Ctrl+C again to force exit.");
    }

    let chrome_path = Arc::new(config.chrome_path.clone());
    let output_dir = Arc::new(config.output_dir.clone());
    let crash_dir = Arc::new(config.crash_dir.clone());
    let verbose = config.verbose;
    let virtual_time_budget = config.virtual_time_budget;
    let buf_size = config.buf_size;
    let shutdown = config.shutdown.clone();

    // Shared counters
    let total_generated = Arc::new(AtomicU64::new(0));
    let completed = Arc::new(AtomicU64::new(0));
    let crash_count = Arc::new(AtomicU64::new(0));
    let unique_crash_count = Arc::new(AtomicU64::new(0));
    let timeout_count = Arc::new(AtomicU64::new(0));
    let asan_found = Arc::new(AtomicBool::new(false));

    let dedup = Arc::new(Mutex::new(CrashDeduplicator::new()));
    let sem = Arc::new(Semaphore::new(jobs));

    let start = Instant::now();

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(4096);
    let mut rng = StdRng::seed_from_u64(actual_seed);
    let mut last_stats = Instant::now();

    let mut i: u64 = 0;
    loop {
        // Check shutdown signal
        if shutdown.load(Ordering::Relaxed) > 0 {
            tracing::info!("Shutdown signal received — stopping generation, waiting for {} in-flight tasks...", handles.len());
            break;
        }

        // Check iteration limit (if not infinite)
        if !infinite && i >= config.iterations {
            break;
        }

        let iter_seed = rng.next_u64();

        // Generate HTML
        let html = {
            let mut iter_rng = StdRng::seed_from_u64(iter_seed);
            let actual_buf_size = buf_size + (iter_rng.next_u32() as usize % (buf_size / 2));
            let mut buf = vec![0u8; actual_buf_size];
            iter_rng.fill_bytes(&mut buf);

            let mut u = Unstructured::new(&buf);
            match FuzzProgram::arbitrary(&mut u) {
                Ok(program) => serialize(
                    &program.font_faces,
                    &program.css_rules,
                    &program.dom,
                    &program.script,
                    &program.keyframes,
                    &program.at_rules,
                ),
                Err(_) => {
                    i += 1;
                    continue;
                }
            }
        };

        let html_len = html.len();
        total_generated.fetch_add(1, Ordering::Relaxed);

        // Write HTML to disk
        let filepath = output_dir.join(format!("test{}.html", i));
        let abs_path = std::fs::canonicalize(&*output_dir)
            .unwrap_or_else(|_| output_dir.to_path_buf())
            .join(format!("test{}.html", i));
        if let Err(e) = tokio::fs::write(&filepath, &html).await {
            tracing::warn!("Failed to write test file: {}", e);
            i += 1;
            continue;
        }

        if verbose {
            tracing::debug!("[iter {}] {} bytes (iter_seed={})", i, html_len, iter_seed);
        }

        // Clone for async task (prefixed t_ to not shadow outer Arcs)
        let t_sem = sem.clone();
        let t_chrome = chrome_path.clone();
        let t_crash_dir = crash_dir.clone();
        let t_completed = completed.clone();
        let t_crash_count = crash_count.clone();
        let t_unique_crash = unique_crash_count.clone();
        let t_timeout_count = timeout_count.clone();
        let t_asan_found = asan_found.clone();
        let t_dedup = dedup.clone();
        let seed_info = format!("master={},iter={}", actual_seed, iter_seed);

        let handle = tokio::spawn(async move {
            let _permit = t_sem.acquire().await.unwrap();

            let result = cdp::run_testcase(&t_chrome, &abs_path, timeout, virtual_time_budget).await;

            match result {
                TestResult::Ok => {
                    let _ = tokio::fs::remove_file(&filepath).await;
                }
                TestResult::Crash { log, crash_type, signal, exit_code } => {
                    t_crash_count.fetch_add(1, Ordering::Relaxed);

                    let is_unique = {
                        let mut d = t_dedup.lock().unwrap();
                        d.is_new(&log)
                    };

                    if is_unique {
                        let unique_idx = t_unique_crash.fetch_add(1, Ordering::Relaxed);

                        let asan_error_type = cdp::extract_asan_error_type(&log);

                        let crash_html = t_crash_dir.join(format!("crash-{}.html", unique_idx));
                        let _ = tokio::fs::copy(&filepath, &crash_html).await;

                        let crash_log_path = t_crash_dir.join(format!("crash-{}.log", unique_idx));
                        let _ = tokio::fs::write(&crash_log_path, &log).await;

                        let metadata = build_crash_metadata(
                            i, crash_type, signal, exit_code,
                            html_len, &seed_info, asan_error_type,
                        );
                        let meta_path = t_crash_dir.join(format!("crash-{}.json", unique_idx));
                        let _ = tokio::fs::write(&meta_path, &metadata).await;

                        if crash_type.is_sanitizer() {
                            let error_desc = asan_error_type.unwrap_or(crash_type.as_str());
                            tracing::error!(
                                "!!! {} #{} at iter {} — {} — saved crash-{}.html",
                                crash_type.as_str(), unique_idx, i, error_desc, unique_idx,
                            );
                            for line in log.lines().take(20) {
                                tracing::error!("  {}", line);
                            }
                            t_asan_found.store(true, Ordering::Relaxed);
                        } else {
                            tracing::warn!(
                                "{} crash #{} at iter {} — saved crash-{}.html",
                                crash_type.as_str(), unique_idx, i, unique_idx,
                            );
                        }
                    }

                    let _ = tokio::fs::remove_file(&filepath).await;
                }
                TestResult::Timeout => {
                    t_timeout_count.fetch_add(1, Ordering::Relaxed);
                    let _ = tokio::fs::remove_file(&filepath).await;
                }
            }

            t_completed.fetch_add(1, Ordering::Relaxed);
        });

        handles.push(handle);

        // Live stats every 5 seconds
        if last_stats.elapsed() >= Duration::from_secs(5) {
            let done = completed.load(Ordering::Relaxed);
            let elapsed = start.elapsed();
            let rate = done as f64 / elapsed.as_secs_f64().max(0.001);
            let uniq = unique_crash_count.load(Ordering::Relaxed);
            let total_c = crash_count.load(Ordering::Relaxed);
            let to = timeout_count.load(Ordering::Relaxed);
            let gen = total_generated.load(Ordering::Relaxed);
            let inflight = gen.saturating_sub(done);

            tracing::info!(
                "[{}] exec: {} | {:.1}/s | crashes: {} unique ({} total) | timeouts: {} | in-flight: {}",
                format_duration(elapsed),
                done, rate, uniq, total_c, to, inflight,
            );
            last_stats = Instant::now();
        }

        // Drain completed handles periodically to avoid unbounded growth
        if handles.len() > 8192 {
            let mut remaining = Vec::with_capacity(4096);
            for h in handles.drain(..) {
                if h.is_finished() {
                    let _ = h.await;
                } else {
                    remaining.push(h);
                }
            }
            handles = remaining;
        }

        i += 1;
    }

    // Wait for all in-flight tasks (graceful shutdown)
    let inflight = handles.len();
    if inflight > 0 {
        tracing::info!("Waiting for {} in-flight tasks to complete...", inflight);
        let mut done_count = 0;
        for h in handles {
            // Check for force exit while waiting
            if shutdown.load(Ordering::Relaxed) > 1 {
                tracing::warn!("Force exit — aborting remaining tasks");
                break;
            }
            let _ = h.await;
            done_count += 1;
            if done_count % 100 == 0 {
                tracing::info!("  ... {}/{} tasks completed", done_count, inflight);
            }
        }
    }

    // Final stats
    let elapsed = start.elapsed();
    let done = completed.load(Ordering::Relaxed);
    let unique_crashes = unique_crash_count.load(Ordering::Relaxed);
    let crashes = crash_count.load(Ordering::Relaxed);
    let timeouts = timeout_count.load(Ordering::Relaxed);

    tracing::info!("=== oxiom — final stats ===");
    tracing::info!(
        "runtime: {} | executions: {} ({:.1}/s)",
        format_duration(elapsed),
        done,
        done as f64 / elapsed.as_secs_f64().max(0.001),
    );
    tracing::info!(
        "crashes: {} unique, {} total | timeouts: {} | seed: {}",
        unique_crashes, crashes, timeouts, actual_seed,
    );

    if asan_found.load(Ordering::Relaxed) {
        tracing::error!(
            "SANITIZER CRASHES FOUND — {} unique reproducers in {}",
            unique_crashes,
            config.crash_dir.display()
        );
    }

    Ok(())
}
