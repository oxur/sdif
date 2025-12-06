//! Terminal output formatting utilities.

use colored::Colorize;
use std::io::{self, Write};

/// Print an error message to stderr.
pub fn print_error(err: &anyhow::Error) {
    eprintln!("{}: {}", "error".red().bold(), err);

    // Print cause chain
    for cause in err.chain().skip(1) {
        eprintln!("  {}: {}", "caused by".red(), cause);
    }
}

/// Print a warning message to stderr.
pub fn print_warning(msg: &str) {
    eprintln!("{}: {}", "warning".yellow().bold(), msg);
}

/// Print an info message to stdout (respects quiet mode).
pub fn print_info(msg: &str, quiet: bool) {
    if !quiet {
        println!("{}", msg);
    }
}

/// Print a success message.
pub fn print_success(msg: &str, quiet: bool) {
    if !quiet {
        println!("{}: {}", "success".green().bold(), msg);
    }
}

/// Print a verbose message (only in verbose mode).
pub fn print_verbose(msg: &str, verbose: bool) {
    if verbose {
        println!("{}: {}", "info".blue(), msg);
    }
}

/// Print a header line.
pub fn print_header(title: &str) {
    println!("\n{}", title.bold().underline());
}

/// Print a key-value pair.
pub fn print_kv(key: &str, value: &str, indent: usize) {
    let padding = " ".repeat(indent);
    println!("{}{}: {}", padding, key.dimmed(), value);
}

/// Print a separator line.
pub fn print_separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Format a number with thousands separators.
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }

    result
}

/// Format a duration in seconds to a human-readable string.
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let mins = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.1}s", mins, secs)
    }
}

/// Format file size in human-readable form.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Create a simple progress reporter for verbose mode.
pub struct ProgressReporter {
    total: usize,
    current: usize,
    last_percent: usize,
    verbose: bool,
}

impl ProgressReporter {
    pub fn new(total: usize, verbose: bool) -> Self {
        ProgressReporter {
            total,
            current: 0,
            last_percent: 0,
            verbose,
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;

        if self.verbose && self.total > 0 {
            let percent = (self.current * 100) / self.total;

            // Only print at 10% intervals
            if percent >= self.last_percent + 10 {
                self.last_percent = percent;
                eprint!(
                    "\r{}: {}% ({}/{} frames)",
                    "progress".blue(),
                    percent,
                    self.current,
                    self.total
                );
                io::stderr().flush().ok();
            }
        }
    }

    pub fn finish(&self) {
        if self.verbose && self.total > 0 {
            eprintln!(
                "\r{}: 100% ({}/{} frames)",
                "progress".blue(),
                self.total,
                self.total
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.5), "500ms");
        assert_eq!(format_duration(1.5), "1.50s");
        assert_eq!(format_duration(90.0), "1m 30.0s");
    }
}
