//! Progress Reporter
//!
//! Provides detailed progress reporting for long-running executions.
//! Supports multiple output formats and real-time updates.

use std::fmt::{Display, Write as FmtWrite};
use std::io::Write;
use std::time::{Duration, Instant};

use crate::signals::Status;

/// Progress reporter for execution tracking
pub struct ProgressReporter {
    /// Start time
    start: Instant,

    /// Total steps
    total_steps: usize,

    /// Current step
    current_step: usize,

    /// Current step name
    step_name: String,

    /// Output format
    format: ProgressFormat,

    /// Verbose mode
    verbose: bool,

    /// Use colors
    colors: bool,

    /// Last progress percentage
    last_progress: u8,

    /// Buffer for output
    buffer: String,
}

impl ProgressReporter {
    /// Create new reporter
    pub fn new(total_steps: usize) -> Self {
        Self {
            start: Instant::now(),
            total_steps,
            current_step: 0,
            step_name: String::new(),
            format: ProgressFormat::Summary,
            verbose: false,
            colors: true,
            last_progress: 0,
            buffer: String::new(),
        }
    }

    /// Set output format
    pub fn with_format(mut self, format: ProgressFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable/disable colors
    pub fn colors(mut self, colors: bool) -> Self {
        self.colors = colors;
        self
    }

    /// Start progress reporting
    pub fn start(&mut self, step_name: &str) {
        self.current_step = 1;
        self.step_name = step_name.to_string();
        self.last_progress = 0;

        if self.verbose {
            self.println(&self.format_header("Starting", step_name));
        }
    }

    /// Update progress
    pub fn update(&mut self, message: &str, progress: u8) {
        let elapsed = self.elapsed();

        if progress != self.last_progress || self.verbose {
            self.last_progress = progress;

            match self.format {
                ProgressFormat::Summary => {
                    self.println(&self.format_summary(message, progress, elapsed));
                }
                ProgressFormat::Detailed => {
                    self.println(&self.format_detailed(message, progress, elapsed));
                }
                ProgressFormat::Minimal => {
                    if progress % 10 == 0 || progress == 100 {
                        self.println(&self.format_summary(message, progress, elapsed));
                    }
                }
                ProgressFormat::Json => {
                    self.println(&self.format_json(message, progress, elapsed));
                }
            }
        }
    }

    /// Report step completion
    pub fn complete_step(&mut self, step_name: &str, success: bool) {
        self.current_step += 1;
        let progress = ((self.current_step as f32 / self.total_steps as f32) * 100.0) as u8;
        self.step_name = step_name.to_string();

        let status = if success { "✓" } else { "✗" };
        if self.colors && success {
            self.println(&format!("{} Completed: {}", status, step_name));
        } else {
            self.println(&format!("{} Completed: {}", status, step_name));
        }

        self.update(step_name, progress);
    }

    /// Finish progress reporting
    pub fn finish(&mut self, message: &str, success: bool) {
        let elapsed = self.elapsed();

        let status = if success {
            self.green("✓")
        } else {
            self.red("✗")
        };

        self.println("");
        self.println(&format!(
            "{} {} (elapsed: {})",
            status,
            message,
            Self::format_duration(elapsed)
        ));

        if self.verbose {
            self.println(&self.format_summary(message, 100, elapsed));
        }
    }

    /// Report an error
    pub fn error(&mut self, message: &str) {
        if self.colors {
            self.println(&format!("{} Error: {}", self.red("✗"), message));
        } else {
            self.println(&format!("X Error: {}", message));
        }
    }

    /// Report a warning
    pub fn warning(&mut self, message: &str) {
        if self.colors {
            self.println(&format!("{} Warning: {}", self.yellow("!"), message));
        } else {
            self.println(&format!("! Warning: {}", message));
        }
    }

    /// Report info
    pub fn info(&mut self, message: &str) {
        if self.verbose {
            self.println(&format!("  Info: {}", message));
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Format elapsed time as string
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs >= 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{:.1}s", secs as f64 + duration.subsec_nanos() as f64 / 1e9)
        }
    }

    /// Format progress bar
    fn format_progress_bar(progress: u8, width: usize) -> String {
        let filled = (progress as usize * width) / 100;
        let empty = width.saturating_sub(filled);

        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    }

    fn format_header(&self, action: &str, step: &str) -> String {
        format!("=== {}: {} ===", action, step)
    }

    fn format_summary(&self, message: &str, progress: u8, elapsed: Duration) -> String {
        let bar = Self::format_progress_bar(progress, 20);
        let elapsed_str = Self::format_duration(elapsed);
        format!("{} {:3}% {} | {}", bar, progress, elapsed_str, message)
    }

    fn format_detailed(&self, message: &str, progress: u8, elapsed: Duration) -> String {
        let bar = Self::format_progress_bar(progress, 30);
        let elapsed_str = Self::format_duration(elapsed);
        let step_info = format!("[{}/{}]", self.current_step, self.total_steps);
        format!(
            "{} {:3}% {} {} | {}",
            bar, progress, step_info, elapsed_str, message
        )
    }

    fn format_json(&self, message: &str, progress: u8, elapsed: Duration) -> String {
        let elapsed_ms = elapsed.as_millis();
        format!(
            r#"{{"progress":{},"elapsed_ms":{},"message":"{}","step":{}}}"#,
            progress, elapsed_ms, message, self.current_step
        )
    }

    fn println(&mut self, line: &str) {
        let _ = writeln!(&mut self.buffer, "{}", line);
        let _ = std::io::stdout().write_all(self.buffer.as_bytes());
        self.buffer.clear();
    }

    fn green(&self, s: &str) -> String {
        if self.colors {
            format!("\x1b[32m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    fn red(&self, s: &str) -> String {
        if self.colors {
            format!("\x1b[31m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }

    fn yellow(&self, s: &str) -> String {
        if self.colors {
            format!("\x1b[33m{}\x1b[0m", s)
        } else {
            s.to_string()
        }
    }
}

/// Output format for progress
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressFormat {
    /// Single-line progress with bar
    Summary,

    /// Multi-line detailed progress
    Detailed,

    /// Minimal output (only milestones)
    Minimal,

    /// JSON lines output
    Json,
}

impl Default for ProgressFormat {
    fn default() -> Self {
        Self::Summary
    }
}

/// Progress callback for integration with execution
pub struct ProgressCallback {
    /// Reporter instance
    reporter: Option<ProgressReporter>,
}

impl ProgressCallback {
    /// Create new callback
    pub fn new(total_steps: usize) -> Self {
        Self {
            reporter: Some(ProgressReporter::new(total_steps)),
        }
    }

    /// Set format
    pub fn with_format(mut self, format: ProgressFormat) -> Self {
        if let Some(ref mut r) = self.reporter {
            r.format = format;
        }
        self
    }

    /// Set verbose
    pub fn verbose(mut self, verbose: bool) -> Self {
        if let Some(ref mut r) = self.reporter {
            r.verbose = verbose;
        }
        self
    }

    /// Get reporter reference
    pub fn reporter(&mut self) -> Option<&mut ProgressReporter> {
        self.reporter.as_mut()
    }

    /// Convert to status signal
    pub fn to_status(&self, package_id: &str) -> Status {
        if let Some(ref r) = self.reporter {
            let progress = ((r.current_step as f32 / r.total_steps as f32) * 100.0) as u8;
            Status::progress(
                package_id,
                &r.step_name,
                progress.min(100),
            )
        } else {
            Status::started(package_id)
        }
    }
}

impl Display for ProgressReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let progress = ((self.current_step as f32 / self.total_steps as f32) * 100.0) as u8;
        let elapsed = Self::format_duration(self.elapsed());
        let bar = Self::format_progress_bar(progress, 20);

        write!(
            f,
            "{} {:3}% | {} | Step {}/{}: {}",
            bar,
            progress,
            elapsed,
            self.current_step,
            self.total_steps,
            self.step_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reporter_creation() {
        let reporter = ProgressReporter::new(10);
        assert_eq!(reporter.total_steps, 10);
    }

    #[test]
    fn progress_reporter_start() {
        let mut reporter = ProgressReporter::new(5);
        reporter.start("step-1");
        assert_eq!(reporter.current_step, 1);
    }

    #[test]
    fn progress_reporter_update() {
        let mut reporter = ProgressReporter::new(10);
        reporter.start("test");
        reporter.update("Testing...", 50);
        assert_eq!(reporter.last_progress, 50);
    }

    #[test]
    fn progress_format_duration() {
        let duration = Duration::from_secs(65);
        assert_eq!(ProgressReporter::format_duration(duration), "1m 5s");

        let duration = Duration::from_millis(1500);
        assert!(ProgressReporter::format_duration(duration).contains("s"));
    }

    #[test]
    fn progress_format_bar() {
        let bar = ProgressReporter::format_progress_bar(50, 10);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert!(bar.contains('='));
    }

    #[test]
    fn progress_callback_to_status() {
        let mut callback = ProgressCallback::new(5);
        let status = callback.to_status("pkg-001");
        assert_eq!(status.package_id, "pkg-001");
    }

    #[test]
    fn progress_format_variants() {
        assert_eq!(ProgressFormat::Summary as u8, 0);
        assert_eq!(ProgressFormat::Detailed as u8, 1);
        assert_eq!(ProgressFormat::Minimal as u8, 2);
        assert_eq!(ProgressFormat::Json as u8, 3);
    }
}
