//! Command-line argument definitions using clap derive macros.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Convert MATLAB/Octave .mat files to SDIF format.
///
/// mat2sdif reads numeric arrays from MAT files and writes them as
/// time-stamped SDIF frames, suitable for use with Max/MSP, AudioSculpt,
/// and other SDIF-compatible software.
#[derive(Parser, Debug)]
#[command(name = "mat2sdif")]
#[command(author, version, about, long_about = None)]
#[command(after_help = EXAMPLES)]
pub struct Args {
    /// Input .mat file
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output .sdif file (omit for --list mode)
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    // ========================================================================
    // Mode Selection
    // ========================================================================
    /// List variables in the MAT file and exit
    #[arg(short, long)]
    pub list: bool,

    /// Validate conversion without writing output
    #[arg(long)]
    pub dry_run: bool,

    // ========================================================================
    // Variable Selection
    // ========================================================================
    /// Variable containing the time vector
    ///
    /// If not specified, mat2sdif will attempt to auto-detect a suitable
    /// time vector based on variable name and data characteristics.
    #[arg(short = 't', long = "time-var", value_name = "NAME")]
    pub time_var: Option<String>,

    /// Variable containing the data matrix
    ///
    /// If not specified, mat2sdif will attempt to auto-detect a suitable
    /// data variable (2D numeric array that isn't the time vector).
    #[arg(short = 'd', long = "data-var", value_name = "NAME")]
    pub data_var: Option<String>,

    // ========================================================================
    // SDIF Configuration
    // ========================================================================
    /// SDIF frame type signature (4 characters)
    #[arg(short = 'f', long, value_name = "SIG", default_value = "1TRC")]
    pub frame_type: String,

    /// SDIF matrix type signature (4 characters)
    #[arg(short = 'm', long, value_name = "SIG", default_value = "1TRC")]
    pub matrix_type: String,

    /// Column names for the matrix (comma-separated)
    ///
    /// Default depends on frame type:
    /// - 1TRC/1HRM: Index,Frequency,Amplitude,Phase
    /// - 1FQ0: Frequency,Confidence
    #[arg(short = 'c', long, value_name = "NAMES", value_delimiter = ',')]
    pub columns: Option<Vec<String>>,

    /// Stream ID for output frames
    #[arg(long, value_name = "ID", default_value = "0")]
    pub stream_id: u32,

    // ========================================================================
    // Max/MSP Compatibility
    // ========================================================================
    /// Maximum partials per frame (for Max/MSP compatibility)
    ///
    /// Modern CNMAT externals support up to 1024 partials.
    /// Set to 256 for legacy compatibility.
    /// Set to 0 to disable limiting.
    #[arg(long, value_name = "N", default_value = "1024")]
    pub max_partials: usize,

    /// Validate Max/MSP compatibility and warn about issues
    #[arg(long)]
    pub max_compat: bool,

    // ========================================================================
    // Data Handling
    // ========================================================================
    /// Transpose the data matrix (swap rows and columns)
    ///
    /// Use this if your data has time as columns instead of rows.
    #[arg(long)]
    pub transpose: bool,

    /// How to handle complex numbers in the data
    #[arg(long, value_enum, default_value = "magnitude")]
    pub complex_mode: ComplexModeArg,

    // ========================================================================
    // Output Control
    // ========================================================================
    /// Show detailed progress and information
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all non-error output
    #[arg(short, long)]
    pub quiet: bool,

    /// Force overwrite of existing output file
    #[arg(long)]
    pub force: bool,
}

/// How to handle complex numbers in MAT data.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ComplexModeArg {
    /// Use only the real part
    Real,
    /// Convert to magnitude (absolute value)
    Magnitude,
    /// Output magnitude and phase as separate columns
    MagPhase,
    /// Output real and imaginary as separate columns
    ReIm,
}

impl Args {
    /// Validate argument combinations.
    pub fn validate(&self) -> Result<(), String> {
        // List mode doesn't need output file
        if self.list {
            return Ok(());
        }

        // Conversion modes need output file
        if self.output.is_none() && !self.dry_run {
            return Err("Output file is required (or use --list or --dry-run)".to_string());
        }

        // Validate signature lengths
        if self.frame_type.len() != 4 {
            return Err(format!(
                "Frame type must be exactly 4 characters, got '{}'",
                self.frame_type
            ));
        }

        if self.matrix_type.len() != 4 {
            return Err(format!(
                "Matrix type must be exactly 4 characters, got '{}'",
                self.matrix_type
            ));
        }

        // Check input file exists
        if !self.input.exists() {
            return Err(format!(
                "Input file not found: {}",
                self.input.display()
            ));
        }

        // Check output doesn't exist (unless --force)
        if let Some(ref output) = self.output {
            if output.exists() && !self.force && !self.dry_run {
                return Err(format!(
                    "Output file already exists: {} (use --force to overwrite)",
                    output.display()
                ));
            }
        }

        // Quiet and verbose are mutually exclusive
        if self.quiet && self.verbose {
            return Err("Cannot use both --quiet and --verbose".to_string());
        }

        Ok(())
    }

    /// Get default column names based on frame type.
    pub fn get_columns(&self) -> Vec<String> {
        if let Some(ref cols) = self.columns {
            return cols.clone();
        }

        // Defaults based on frame type
        match self.frame_type.as_str() {
            "1TRC" | "1HRM" => vec![
                "Index".to_string(),
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "Phase".to_string(),
            ],
            "1FQ0" => vec!["Frequency".to_string(), "Confidence".to_string()],
            "1RES" => vec![
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "DecayRate".to_string(),
                "Phase".to_string(),
            ],
            _ => vec![
                "Col1".to_string(),
                "Col2".to_string(),
                "Col3".to_string(),
                "Col4".to_string(),
            ],
        }
    }
}

/// Example usage shown in --help.
const EXAMPLES: &str = r#"
EXAMPLES:
    # List variables in a MAT file
    mat2sdif --list analysis.mat

    # Basic conversion with auto-detection
    mat2sdif analysis.mat output.sdif

    # Specify time and data variables explicitly
    mat2sdif analysis.mat output.sdif -t time -d partials

    # Convert with custom column names
    mat2sdif analysis.mat output.sdif -c "Index,Freq,Amp,Phase"

    # Convert F0 data
    mat2sdif pitch.mat f0.sdif -f 1FQ0 -m 1FQ0 -c "Frequency,Confidence"

    # Validate without writing (dry run)
    mat2sdif --dry-run analysis.mat output.sdif

    # Force overwrite and show progress
    mat2sdif -v --force analysis.mat output.sdif

    # Legacy Max compatibility (256 partial limit)
    mat2sdif --max-partials 256 analysis.mat output.sdif
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_columns_1trc() {
        let args = Args {
            input: PathBuf::from("test.mat"),
            output: Some(PathBuf::from("test.sdif")),
            list: false,
            dry_run: false,
            time_var: None,
            data_var: None,
            frame_type: "1TRC".to_string(),
            matrix_type: "1TRC".to_string(),
            columns: None,
            stream_id: 0,
            max_partials: 1024,
            max_compat: false,
            transpose: false,
            complex_mode: ComplexModeArg::Magnitude,
            verbose: false,
            quiet: false,
            force: false,
        };

        let cols = args.get_columns();
        assert_eq!(cols.len(), 4);
        assert_eq!(cols[0], "Index");
    }

    #[test]
    fn test_default_columns_1fq0() {
        let args = Args {
            input: PathBuf::from("test.mat"),
            output: Some(PathBuf::from("test.sdif")),
            list: false,
            dry_run: false,
            time_var: None,
            data_var: None,
            frame_type: "1FQ0".to_string(),
            matrix_type: "1FQ0".to_string(),
            columns: None,
            stream_id: 0,
            max_partials: 1024,
            max_compat: false,
            transpose: false,
            complex_mode: ComplexModeArg::Magnitude,
            verbose: false,
            quiet: false,
            force: false,
        };

        let cols = args.get_columns();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], "Frequency");
    }
}
