//! Max/MSP compatibility validation.
//!
//! This module provides checks to ensure generated SDIF files will work
//! correctly with Max/MSP and the CNMAT SDIF externals.

use colored::Colorize;

use sdif_rs::MatToSdifConverter;

use crate::cli::Args;
use crate::output;

/// Max-compatible frame types.
const MAX_FRAME_TYPES: &[&str] = &["1TRC", "1HRM", "1FQ0", "1RES"];

/// Modern CNMAT partial limit.
const MODERN_PARTIAL_LIMIT: usize = 1024;

/// Legacy CNMAT partial limit.
const LEGACY_PARTIAL_LIMIT: usize = 256;

/// Validate configuration for Max compatibility.
///
/// Returns Ok if compatible, or an error with explanation if not.
pub fn validate_config(args: &Args, converter: &MatToSdifConverter) -> anyhow::Result<()> {
    let warnings = check_all(args, converter);

    for warning in &warnings {
        output::print_warning(warning);
    }

    Ok(())
}

/// Run all compatibility checks and return warnings.
pub fn check_all(args: &Args, converter: &MatToSdifConverter) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check frame type
    if let Some(w) = check_frame_type(&args.frame_type) {
        warnings.push(w);
    }

    // Check partial limit
    if let Some(w) = check_partial_limit(args.max_partials, converter.cols_per_frame()) {
        warnings.push(w);
    }

    // Check column count for specific frame types
    if let Some(w) = check_column_count(&args.frame_type, &args.get_columns()) {
        warnings.push(w);
    }

    // Check time range
    let (start, end) = converter.time_range();
    if let Some(w) = check_time_range(start, end) {
        warnings.push(w);
    }

    warnings
}

/// Check if frame type is Max-compatible.
fn check_frame_type(frame_type: &str) -> Option<String> {
    if !MAX_FRAME_TYPES.contains(&frame_type) {
        Some(format!(
            "Frame type '{}' may not be supported by all Max externals. \
             Standard types are: {}",
            frame_type,
            MAX_FRAME_TYPES.join(", ")
        ))
    } else {
        None
    }
}

/// Check partial limit against Max constraints.
fn check_partial_limit(limit: usize, _cols: usize) -> Option<String> {
    if limit == 0 {
        return Some(
            "No partial limit set. Max/MSP externals have limits \
             (1024 modern, 256 legacy). Consider setting --max-partials."
                .to_string()
        );
    }

    if limit > MODERN_PARTIAL_LIMIT {
        return Some(format!(
            "Partial limit {} exceeds Max/MSP limit of {}. \
             Frames may be truncated during playback.",
            limit, MODERN_PARTIAL_LIMIT
        ));
    }

    if limit > LEGACY_PARTIAL_LIMIT {
        return Some(format!(
            "Partial limit {} exceeds legacy Max limit of {}. \
             May not work with older CNMAT externals.",
            limit, LEGACY_PARTIAL_LIMIT
        ));
    }

    None
}

/// Check column count matches expected for frame type.
fn check_column_count(frame_type: &str, columns: &[String]) -> Option<String> {
    let expected = match frame_type {
        "1TRC" | "1HRM" => 4, // Index, Frequency, Amplitude, Phase
        "1FQ0" => 2,          // Frequency, Confidence
        "1RES" => 4,          // Frequency, Amplitude, DecayRate, Phase
        _ => return None,     // Unknown type, skip check
    };

    if columns.len() != expected {
        Some(format!(
            "Frame type '{}' typically has {} columns, but {} provided. \
             This may cause issues with some software.",
            frame_type, expected, columns.len()
        ))
    } else {
        None
    }
}

/// Check time range is reasonable.
fn check_time_range(start: f64, end: f64) -> Option<String> {
    if start < 0.0 {
        return Some(format!(
            "Negative start time ({:.3}s) may cause issues. \
             Consider normalizing to start at 0.",
            start
        ));
    }

    if end > 3600.0 {
        return Some(format!(
            "Duration over 1 hour ({:.1}s). \
             Very long files may have performance issues.",
            end - start
        ));
    }

    None
}

/// Detailed compatibility report for verbose mode.
pub fn print_compatibility_report(args: &Args, converter: &MatToSdifConverter) {
    output::print_header("Max/MSP Compatibility Report");

    // Frame type
    let frame_ok = MAX_FRAME_TYPES.contains(&args.frame_type.as_str());
    println!(
        "  Frame type '{}': {}",
        args.frame_type,
        if frame_ok { "✓ Supported".green() } else { "⚠ Non-standard".yellow() }
    );

    // Partial limit
    let partial_status = if args.max_partials == 0 {
        "⚠ No limit".yellow().to_string()
    } else if args.max_partials <= LEGACY_PARTIAL_LIMIT {
        "✓ Legacy compatible".green().to_string()
    } else if args.max_partials <= MODERN_PARTIAL_LIMIT {
        "✓ Modern compatible".green().to_string()
    } else {
        "✗ Exceeds limit".red().to_string()
    };
    println!("  Partial limit {}: {}", args.max_partials, partial_status);

    // Column count
    let cols = args.get_columns();
    println!("  Columns: {} ({:?})", cols.len(), cols);

    // Time range
    let (start, end) = converter.time_range();
    let duration = end - start;
    println!(
        "  Time range: {:.3}s to {:.3}s ({:.2}s duration)",
        start, end, duration
    );

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_check() {
        assert!(check_frame_type("1TRC").is_none());
        assert!(check_frame_type("1HRM").is_none());
        assert!(check_frame_type("1FQ0").is_none());
        assert!(check_frame_type("XXXX").is_some());
    }

    #[test]
    fn test_partial_limit_check() {
        assert!(check_partial_limit(256, 4).is_none());
        assert!(check_partial_limit(1024, 4).is_some()); // Warning for > legacy
        assert!(check_partial_limit(2000, 4).is_some()); // Error for > modern
        assert!(check_partial_limit(0, 4).is_some());    // Warning for no limit
    }

    #[test]
    fn test_column_count_check() {
        let cols_4 = vec!["A".into(), "B".into(), "C".into(), "D".into()];
        let cols_2 = vec!["A".into(), "B".into()];

        assert!(check_column_count("1TRC", &cols_4).is_none());
        assert!(check_column_count("1TRC", &cols_2).is_some());
        assert!(check_column_count("1FQ0", &cols_2).is_none());
        assert!(check_column_count("1FQ0", &cols_4).is_some());
    }
}
