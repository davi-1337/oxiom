use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    pub iterations: u64,
    pub timeout_ms: u64,
    pub output_dir: PathBuf,
    pub crash_dir: PathBuf,
    pub jobs: usize,
    pub verbose: bool,
    pub continuous: bool,
    pub virtual_time_budget: u64,
    pub seed: u64,
    pub buf_size: usize,
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

    /// Extract a hash from ASAN stack trace. Strip addresses, keep top 5 frame function names.
    fn hash_crash(&self, log: &str) -> u64 {
        let mut hasher_val: u64 = 0xcbf29ce484222325; // FNV offset
        let mut frame_count = 0;

        for line in log.lines() {
            let trimmed = line.trim();
            // ASAN frames look like: #0 0xABCD in FunctionName file.cc:123
            if trimmed.starts_with('#') && frame_count < 5 {
                // Extract function name: skip address, get the "in <name>" part
                if let Some(in_pos) = trimmed.find(" in ") {
                    let after_in = &trimmed[in_pos + 4..];
                    let func_name = after_in.split_whitespace().next().unwrap_or(after_in);
                    // FNV-1a hash
                    for b in func_name.bytes() {
                        hasher_val ^= b as u64;
                        hasher_val = hasher_val.wrapping_mul(0x100000001b3);
                    }
                    frame_count += 1;
                }
            }

            // Also hash the error type line (e.g., "heap-use-after-free")
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
                    hasher_val ^= b as u64;
                    hasher_val = hasher_val.wrapping_mul(0x100000001b3);
                }
            }
        }

        hasher_val
    }

    /// Returns true if this is a new unique crash.
    fn is_new(&mut self, log: &str) -> bool {
        let hash = self.hash_crash(log);
        self.seen.insert(hash)
    }
}

/// Build crash metadata JSON for perfect reproducibility and analysis.
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
        chrono_now(),
    )
}

/// Simple timestamp without chrono dependency.
fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", dur.as_secs())
}

pub async fn run(config: RunnerConfig) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&config.output_dir).await?;
    tokio::fs::create_dir_all(&config.crash_dir).await?;

    let timeout = Duration::from_millis(config.timeout_ms);
    let jobs = config.jobs;

    // Determine actual seed for logging
    let actual_seed = if config.seed != 0 {
        config.seed
    } else {
        // Use entropy but log the seed for reproducibility
        rand::random::<u64>()
    };

    tracing::info!(
        "Starting fuzzer: {} iterations, {}ms timeout, {} parallel jobs, {}ms vt-budget, buf={}B, seed={}",
        if config.continuous { "infinite".to_string() } else { config.iterations.to_string() },
        config.timeout_ms,
        jobs,
        config.virtual_time_budget,
        config.buf_size,
        actual_seed,
    );

    let chrome_path = Arc::new(config.chrome_path.clone());
    let output_dir = Arc::new(config.output_dir.clone());
    let crash_dir = Arc::new(config.crash_dir.clone());
    let verbose = config.verbose;
    let virtual_time_budget = config.virtual_time_budget;
    let buf_size = config.buf_size;

    // Shared counters
    let completed = Arc::new(AtomicU64::new(0));
    let crash_count = Arc::new(AtomicU64::new(0));
    let unique_crash_count = Arc::new(AtomicU64::new(0));
    let timeout_count = Arc::new(AtomicU64::new(0));
    let asan_found = Arc::new(AtomicBool::new(false));

    // Crash deduplicator
    let dedup = Arc::new(Mutex::new(CrashDeduplicator::new()));

    // Semaphore limits parallel Chrome processes
    let sem = Arc::new(Semaphore::new(jobs));

    let start = Instant::now();

    let mut handles = Vec::with_capacity(config.iterations.min(10000) as usize);

    let mut rng = StdRng::seed_from_u64(actual_seed);

    let mut i: u64 = 0;
    loop {
        if !config.continuous && i >= config.iterations {
            break;
        }

        // Capture the RNG state for this iteration (for reproducibility)
        let iter_seed = rng.next_u64();

        // Generate HTML synchronously (fast, CPU-bound)
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

        // Write HTML to disk
        let filepath = output_dir.join(format!("test{}.html", i));
        let abs_path = std::fs::canonicalize(&*output_dir)
            .unwrap_or_else(|_| output_dir.to_path_buf())
            .join(format!("test{}.html", i));
        tokio::fs::write(&filepath, &html).await?;

        if verbose {
            tracing::debug!("[iter {}] generated {} bytes HTML (iter_seed={})", i, html_len, iter_seed);
        }

        // Clone Arcs for the async task
        let sem = sem.clone();
        let chrome_path = chrome_path.clone();
        let crash_dir = crash_dir.clone();
        let completed = completed.clone();
        let crash_count = crash_count.clone();
        let unique_crash_count = unique_crash_count.clone();
        let timeout_count = timeout_count.clone();
        let asan_found = asan_found.clone();
        let dedup = dedup.clone();
        let iterations = config.iterations;
        let continuous = config.continuous;
        let seed_info = format!("master_seed={},iter_seed={}", actual_seed, iter_seed);

        let handle = tokio::spawn(async move {
            // Acquire semaphore slot — limits parallelism
            let _permit = sem.acquire().await.unwrap();

            let result = cdp::run_testcase(&chrome_path, &abs_path, timeout, virtual_time_budget).await;

            match result {
                TestResult::Ok => {
                    let _ = tokio::fs::remove_file(&filepath).await;
                }
                TestResult::Crash { log, crash_type, signal, exit_code } => {
                    let total_idx = crash_count.fetch_add(1, Ordering::Relaxed);

                    // Deduplicate crashes
                    let is_unique = {
                        let mut d = dedup.lock().unwrap();
                        d.is_new(&log)
                    };

                    if is_unique {
                        let unique_idx = unique_crash_count.fetch_add(1, Ordering::Relaxed);

                        // Extract ASAN error type for metadata
                        let asan_error_type = cdp::extract_asan_error_type(&log);

                        // Save crash HTML reproducer
                        let crash_html = crash_dir.join(format!("{}.html", unique_idx));
                        let _ = tokio::fs::copy(&filepath, &crash_html).await;

                        // Save ASAN/crash log
                        let crash_log_path = crash_dir.join(format!("{}.log", unique_idx));
                        let _ = tokio::fs::write(&crash_log_path, &log).await;

                        // Save crash metadata JSON
                        let metadata = build_crash_metadata(
                            i,
                            crash_type,
                            signal,
                            exit_code,
                            html_len,
                            &seed_info,
                            asan_error_type,
                        );
                        let meta_path = crash_dir.join(format!("{}.json", unique_idx));
                        let _ = tokio::fs::write(&meta_path, &metadata).await;

                        if crash_type.is_sanitizer() {
                            let error_desc = asan_error_type.unwrap_or(crash_type.as_str());
                            tracing::error!(
                                "🔴 {} CRASH #{} at iter {} (total #{}) — {} | saved: {}.html/.log/.json",
                                crash_type.as_str(), unique_idx, i, total_idx,
                                error_desc, unique_idx,
                            );
                            // Print first 25 lines of the crash log
                            for line in log.lines().take(25) {
                                tracing::error!("  {}", line);
                            }
                            asan_found.store(true, Ordering::Relaxed);
                        } else {
                            tracing::warn!(
                                "⚠ {} crash #{} at iter {} — saved: {}.html/.log/.json",
                                crash_type.as_str(), unique_idx, i, unique_idx,
                            );
                        }
                    }

                    let _ = tokio::fs::remove_file(&filepath).await;
                }
                TestResult::Timeout => {
                    timeout_count.fetch_add(1, Ordering::Relaxed);
                    let _ = tokio::fs::remove_file(&filepath).await;
                }
            }

            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

            // Progress every 100 iterations (or every iter if verbose)
            let interval = if verbose { 1 } else { 100 };
            if done % interval == 0 || (!continuous && done == iterations) {
                let elapsed = start.elapsed().as_secs_f64();
                let rate = done as f64 / elapsed;
                tracing::info!(
                    "[{}/{}] {:.1} exec/s | {} unique crashes ({} total) | {} timeouts",
                    done,
                    if continuous { "inf".to_string() } else { iterations.to_string() },
                    rate,
                    unique_crash_count.load(Ordering::Relaxed),
                    crash_count.load(Ordering::Relaxed),
                    timeout_count.load(Ordering::Relaxed),
                );
            }
        });

        handles.push(handle);

        // Periodically drain completed handles to avoid unbounded growth in continuous mode
        if handles.len() > 10000 {
            let mut remaining = Vec::new();
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

    // Wait for all in-flight tasks
    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed();
    let done = completed.load(Ordering::Relaxed);
    let crashes = crash_count.load(Ordering::Relaxed);
    let unique_crashes = unique_crash_count.load(Ordering::Relaxed);
    let timeouts = timeout_count.load(Ordering::Relaxed);

    tracing::info!(
        "Done: {} iterations in {:.1}s ({:.1} exec/s) | {} unique crashes ({} total) | {} timeouts | seed={}",
        done,
        elapsed.as_secs_f64(),
        done as f64 / elapsed.as_secs_f64(),
        unique_crashes,
        crashes,
        timeouts,
        actual_seed,
    );

    if asan_found.load(Ordering::Relaxed) {
        tracing::error!(
            "ASAN crash(es) detected — {} unique reproducers in {}",
            unique_crashes,
            config.crash_dir.display()
        );
    }

    Ok(())
}
