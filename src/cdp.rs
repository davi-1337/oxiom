use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Result of running a single test case through Chrome.
pub enum TestResult {
    /// Page loaded and exited cleanly.
    Ok,
    /// Chrome crashed — contains ASAN/crash log.
    Crash(String),
    /// Timed out.
    Timeout,
}

/// ASAN signature patterns in stderr.
const ASAN_SIGNATURES: &[&str] = &[
    "AddressSanitizer",
    "ERROR: AddressSanitizer",
    "heap-use-after-free",
    "heap-buffer-overflow",
    "stack-buffer-overflow",
    "global-buffer-overflow",
    "use-after-poison",
    "use-after-scope",
    "stack-use-after-return",
    "double-free",
    "alloc-dealloc-mismatch",
    "SEGV on unknown address",
    "attempting free on address",
    "READ of size",
    "WRITE of size",
    "LeakSanitizer",
    "MemorySanitizer",
    "UndefinedBehaviorSanitizer",
    "ThreadSanitizer",
];

/// Check if stderr output contains ASAN crash indicators.
fn find_asan_crash(stderr: &str) -> Option<String> {
    for sig in ASAN_SIGNATURES {
        if stderr.contains(sig) {
            return Some(stderr.to_string());
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

/// Run a single test case: spawn Chrome headless, wait, parse stderr for ASAN.
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
            return TestResult::Crash(format!("Failed to spawn Chrome: {}", e));
        }
    };

    // Wait with timeout
    let wait_result = tokio::time::timeout(timeout, child.wait()).await;

    match wait_result {
        Ok(Ok(status)) => {
            // Read stderr
            let stderr = read_stderr(&mut child).await;

            // Check for ASAN in stderr
            if let Some(asan_log) = find_asan_crash(&stderr) {
                return TestResult::Crash(asan_log);
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
                    return TestResult::Crash(crash_log);
                }
            }

            if !status.success() {
                // Non-zero exit but no signal — could be renderer crash
                let code = status.code().unwrap_or(-1);
                if code != 0 {
                    if !stderr.is_empty() && stderr.len() > 50 {
                        return TestResult::Crash(format!(
                            "Chrome exited with code {}\n\nSTDERR:\n{}",
                            code, stderr
                        ));
                    }
                }
            }

            TestResult::Ok
        }
        Ok(Err(e)) => {
            let stderr = read_stderr(&mut child).await;
            TestResult::Crash(format!("Chrome wait error: {}\n\nSTDERR:\n{}", e, stderr))
        }
        Err(_) => {
            // Timeout — kill the process
            let _ = child.kill().await;
            TestResult::Timeout
        }
    }
}

async fn read_stderr(child: &mut tokio::process::Child) -> String {
    if let Some(mut stderr) = child.stderr.take() {
        let mut buf = Vec::with_capacity(8192);
        // Read up to 64KB of stderr
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            stderr.read_to_end(&mut buf),
        )
        .await;
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
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
        _ => "UNKNOWN",
    }
}
