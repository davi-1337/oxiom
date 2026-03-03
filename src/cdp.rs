use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Result of running a single test case through Chrome.
pub enum TestResult {
    /// Page loaded and exited cleanly.
    Ok,
    /// Chrome crashed — contains ASAN/crash log + crash type classification.
    Crash {
        log: String,
        crash_type: CrashType,
        signal: Option<i32>,
        exit_code: Option<i32>,
    },
    /// Timed out.
    Timeout,
}

/// Classification of the crash for deduplication and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashType {
    /// AddressSanitizer (UAF, buffer overflow, etc.)
    Asan,
    /// MemorySanitizer (uninitialized memory)
    Msan,
    /// UndefinedBehaviorSanitizer
    Ubsan,
    /// ThreadSanitizer (data race)
    Tsan,
    /// LeakSanitizer (memory leak)
    Lsan,
    /// Signal-based crash (SIGSEGV, SIGABRT, etc.) without sanitizer report
    Signal,
    /// Non-zero exit code crash
    ExitCode,
    /// Chrome process error
    ProcessError,
    /// Chromium CHECK/DCHECK failure
    CheckFailure,
    /// Renderer/GPU process crash
    RendererKill,
    /// Out of memory
    Oom,
}

impl CrashType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asan => "ASAN",
            Self::Msan => "MSAN",
            Self::Ubsan => "UBSAN",
            Self::Tsan => "TSAN",
            Self::Lsan => "LSAN",
            Self::Signal => "SIGNAL",
            Self::ExitCode => "EXIT_CODE",
            Self::ProcessError => "PROCESS_ERROR",
            Self::CheckFailure => "CHECK_FAILURE",
            Self::RendererKill => "RENDERER_KILL",
            Self::Oom => "OOM",
        }
    }

    pub fn is_sanitizer(&self) -> bool {
        matches!(self, Self::Asan | Self::Msan | Self::Ubsan | Self::Tsan | Self::Lsan)
    }

    pub fn is_high_value(&self) -> bool {
        matches!(self, Self::Asan | Self::Msan | Self::Ubsan | Self::CheckFailure | Self::Signal)
    }
}

/// ASAN/sanitizer signature patterns in stderr.
const SANITIZER_PATTERNS: &[(&str, CrashType)] = &[
    // CHECK/DCHECK failures (Chromium-specific, real bugs)
    ("Check failed:", CrashType::CheckFailure),
    ("DCHECK failed:", CrashType::CheckFailure),
    ("Fatal error in", CrashType::CheckFailure),  // V8 fatal
    ("[FATAL:", CrashType::CheckFailure),          // Chromium logging
    ("#CRASHED", CrashType::CheckFailure),          // test harness marker
    ("Received signal", CrashType::CheckFailure),   // Chromium signal handler

    // Renderer/GPU process crashes
    ("Renderer process exited", CrashType::RendererKill),
    ("GPU process exited", CrashType::RendererKill),
    ("Renderer process crashed", CrashType::RendererKill),
    ("renderer killed", CrashType::RendererKill),

    // OOM
    ("Out of memory", CrashType::Oom),
    ("Allocation failed", CrashType::Oom),

    // AddressSanitizer patterns (most common)
    ("ERROR: AddressSanitizer:", CrashType::Asan),
    ("AddressSanitizer: heap-use-after-free", CrashType::Asan),
    ("AddressSanitizer: heap-buffer-overflow", CrashType::Asan),
    ("AddressSanitizer: stack-buffer-overflow", CrashType::Asan),
    ("AddressSanitizer: global-buffer-overflow", CrashType::Asan),
    ("AddressSanitizer: use-after-poison", CrashType::Asan),
    ("AddressSanitizer: use-after-scope", CrashType::Asan),
    ("AddressSanitizer: stack-use-after-return", CrashType::Asan),
    ("AddressSanitizer: double-free", CrashType::Asan),
    ("AddressSanitizer: alloc-dealloc-mismatch", CrashType::Asan),
    ("AddressSanitizer: attempting free on address", CrashType::Asan),
    ("AddressSanitizer: SEGV on unknown address", CrashType::Asan),
    ("AddressSanitizer: stack-overflow", CrashType::Asan),
    ("AddressSanitizer: container-overflow", CrashType::Asan),
    ("AddressSanitizer: negative-size-param", CrashType::Asan),
    ("AddressSanitizer: calloc-overflow", CrashType::Asan),
    ("AddressSanitizer: allocator is out of memory", CrashType::Asan),
    ("AddressSanitizer: odr-violation", CrashType::Asan),
    // Bare patterns (ASAN output without prefix)
    ("READ of size", CrashType::Asan),
    ("WRITE of size", CrashType::Asan),
    ("heap-use-after-free on address", CrashType::Asan),
    ("heap-buffer-overflow on address", CrashType::Asan),
    ("SUMMARY: AddressSanitizer:", CrashType::Asan),
    // MemorySanitizer
    ("MemorySanitizer", CrashType::Msan),
    ("WARNING: MemorySanitizer:", CrashType::Msan),
    // UndefinedBehaviorSanitizer
    ("UndefinedBehaviorSanitizer", CrashType::Ubsan),
    ("runtime error:", CrashType::Ubsan),
    // ThreadSanitizer
    ("ThreadSanitizer", CrashType::Tsan),
    ("WARNING: ThreadSanitizer:", CrashType::Tsan),
    // LeakSanitizer
    ("LeakSanitizer", CrashType::Lsan),
    ("ERROR: LeakSanitizer:", CrashType::Lsan),
    ("detected memory leaks", CrashType::Lsan),
];

/// Check if stderr output contains sanitizer crash indicators.
/// Returns the crash type if found.
fn classify_crash(stderr: &str) -> Option<CrashType> {
    // Check patterns in order (most specific first)
    for (pattern, crash_type) in SANITIZER_PATTERNS {
        if stderr.contains(pattern) {
            return Some(*crash_type);
        }
    }
    None
}

/// Extract the ASAN error type from the log (e.g., "heap-use-after-free").
pub fn extract_asan_error_type(log: &str) -> Option<&str> {
    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.contains("ERROR: AddressSanitizer:") {
            // "ERROR: AddressSanitizer: heap-use-after-free on address..."
            if let Some(after) = trimmed.strip_prefix("==").and_then(|s| s.find("ERROR: AddressSanitizer: ").map(|p| &s[p + 24..])) {
                let error_type = after.split_whitespace().next().unwrap_or("unknown");
                return Some(error_type);
            }
            // Fallback: just find after "AddressSanitizer: "
            if let Some(pos) = trimmed.find("AddressSanitizer: ") {
                let after = &trimmed[pos + 18..];
                let error_type = after.split([' ', '\n'].as_ref()).next().unwrap_or("unknown");
                return Some(error_type);
            }
        }
    }
    None
}

/// Build the Chrome command with all headless/fuzzing flags.
fn build_chrome_command(
    chrome_path: &Path,
    html_path: &Path,
    virtual_time_budget: u64,
) -> Command {
    let mut cmd = Command::new(chrome_path);
    cmd.args([
        "--headless",
        "--no-sandbox",
        "--disable-gpu",
        "--disable-software-rasterizer",
        "--disable-dev-shm-usage",
        "--disable-background-networking",
        "--disable-default-apps",
        "--disable-extensions",
        "--disable-sync",
        "--disable-translate",
        "--no-first-run",
        "--disable-hang-monitor",
        "--disable-popup-blocking",
        "--disable-prompt-on-repost",
        "--disable-background-timer-throttling",
        "--disable-renderer-backgrounding",
        "--disable-backgrounding-occluded-windows",
        "--disable-ipc-flooding-protection",
        "--disable-component-update",
        "--disable-domain-reliability",
        "--disable-features=TranslateUI",
        "--disable-site-isolation-trials",
        "--expose-gc",
        "--js-flags=--expose-gc",
        "--allow-file-access-from-files",
        "--run-all-compositor-stages-before-draw",
        "--disable-new-content-rendering-timeout",
        "--window-size=800,600",
    ]);

    cmd.arg(format!("--virtual-time-budget={}", virtual_time_budget));

    let url = format!("file://{}", html_path.display());
    cmd.arg(&url);

    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    // Kill entire process group on drop
    cmd.kill_on_drop(true);

    cmd
}

/// Run a single test case: spawn Chrome headless, read stderr concurrently, check for crashes.
///
/// Key design: stderr is read concurrently with process execution to avoid
/// pipe buffer deadlocks. ASAN output can be very large (100KB+), and if the
/// pipe buffer fills up, Chrome will block on write and never exit.
pub async fn run_testcase(
    chrome_path: &Path,
    html_path: &Path,
    timeout: Duration,
    virtual_time_budget: u64,
) -> TestResult {
    let mut cmd = build_chrome_command(chrome_path, html_path, virtual_time_budget);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return TestResult::Crash {
                log: format!("Failed to spawn Chrome: {}", e),
                crash_type: CrashType::ProcessError,
                signal: None,
                exit_code: None,
            };
        }
    };

    // Take stderr handle immediately and read concurrently with process execution.
    // This prevents pipe buffer deadlock when ASAN produces large output.
    let stderr_handle = child.stderr.take();
    let stderr_task = tokio::spawn(async move {
        if let Some(mut stderr) = stderr_handle {
            let mut buf = Vec::with_capacity(256 * 1024); // 256KB initial capacity
            match tokio::time::timeout(
                Duration::from_secs(30), // generous timeout for stderr reading
                stderr.read_to_end(&mut buf),
            )
            .await
            {
                Ok(Ok(_)) => String::from_utf8_lossy(&buf).to_string(),
                Ok(Err(_)) => String::from_utf8_lossy(&buf).to_string(),
                Err(_) => {
                    // Timeout reading stderr — return what we have
                    String::from_utf8_lossy(&buf).to_string()
                }
            }
        } else {
            String::new()
        }
    });

    // Wait for process with timeout
    let wait_result = tokio::time::timeout(timeout, child.wait()).await;

    // Get stderr output (wait for the concurrent reader to finish)
    let stderr = match tokio::time::timeout(Duration::from_secs(5), stderr_task).await {
        Ok(Ok(s)) => s,
        Ok(Err(_)) => String::new(),
        Err(_) => String::new(),
    };

    match wait_result {
        Ok(Ok(status)) => {
            // Check for sanitizer crash in stderr (highest priority)
            if let Some(crash_type) = classify_crash(&stderr) {
                return TestResult::Crash {
                    log: stderr,
                    crash_type,
                    signal: None,
                    exit_code: status.code(),
                };
            }

            // Check exit code — signals indicate crashes
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(signal) = status.signal() {
                    // SIGSEGV=11, SIGABRT=6, SIGBUS=7, SIGFPE=8, SIGILL=4
                    let crash_log = format!(
                        "Chrome killed by signal {} ({})\n\nSTDERR:\n{}",
                        signal,
                        signal_name(signal),
                        stderr,
                    );
                    return TestResult::Crash {
                        log: crash_log,
                        crash_type: CrashType::Signal,
                        signal: Some(signal),
                        exit_code: None,
                    };
                }
            }

            if !status.success() {
                // Chrome-specific exit codes
                let code = status.code().unwrap_or(-1);
                match code {
                    133 => return TestResult::Crash {
                        log: format!("Chrome renderer crash (exit 133)\n\nSTDERR:\n{}", stderr),
                        crash_type: CrashType::RendererKill,
                        signal: None,
                        exit_code: Some(133),
                    },
                    139 => return TestResult::Crash {
                        log: format!("Chrome SIGSEGV (exit 139)\n\nSTDERR:\n{}", stderr),
                        crash_type: CrashType::Signal,
                        signal: Some(11),
                        exit_code: Some(139),
                    },
                    134 => return TestResult::Crash {
                        log: format!("Chrome SIGABRT (exit 134)\n\nSTDERR:\n{}", stderr),
                        crash_type: CrashType::Signal,
                        signal: Some(6),
                        exit_code: Some(134),
                    },
                    _ => {
                        // Non-zero exit with substantial stderr indicates renderer crash
                        if code != 0 && !stderr.is_empty() && stderr.len() > 50 {
                            return TestResult::Crash {
                                log: format!(
                                    "Chrome exited with code {}\n\nSTDERR:\n{}",
                                    code, stderr
                                ),
                                crash_type: CrashType::ExitCode,
                                signal: None,
                                exit_code: Some(code),
                            };
                        }
                    }
                }
            }

            TestResult::Ok
        }
        Ok(Err(e)) => TestResult::Crash {
            log: format!("Chrome wait error: {}\n\nSTDERR:\n{}", e, stderr),
            crash_type: CrashType::ProcessError,
            signal: None,
            exit_code: None,
        },
        Err(_) => {
            // Timeout — kill the process
            let _ = child.kill().await;
            TestResult::Timeout
        }
    }
}

#[cfg(unix)]
fn signal_name(sig: i32) -> &'static str {
    match sig {
        4 => "SIGILL",
        6 => "SIGABRT",
        7 => "SIGBUS",
        8 => "SIGFPE",
        11 => "SIGSEGV",
        15 => "SIGTERM",
        _ => "UNKNOWN",
    }
}
