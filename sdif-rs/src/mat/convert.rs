//! MAT to SDIF conversion utilities.
//!
//! This module provides [`MatToSdifConverter`] for converting MAT file
//! data to SDIF format, and [`MatToSdifConfig`] for configuration options.

use ndarray::Array1;
use ndarray::Array2;

use crate::error::{Error, Result};
use crate::writer::SdifWriter;
use super::data::MatData;
use super::file::MatFile;

/// Configuration for MAT to SDIF conversion.
///
/// # Example
///
/// ```
/// use sdif_rs::MatToSdifConfig;
///
/// let config = MatToSdifConfig::new()
///     .frame_type("1TRC")
///     .matrix_type("1TRC")
///     .columns(&["Index", "Frequency", "Amplitude", "Phase"]);
/// ```
#[derive(Debug, Clone)]
pub struct MatToSdifConfig {
    /// Name of the time variable (None = auto-detect).
    pub time_variable: Option<String>,

    /// Name of the data variable to convert.
    pub data_variable: Option<String>,

    /// SDIF frame type signature.
    pub frame_type: String,

    /// SDIF matrix type signature.
    pub matrix_type: String,

    /// Column names for the matrix.
    pub columns: Vec<String>,

    /// Maximum partials per frame (for Max/MSP compatibility).
    pub max_partials: Option<usize>,

    /// Whether to transpose the data (swap rows/columns).
    pub transpose: bool,

    /// How to handle complex data.
    pub complex_mode: ComplexMode,

    /// Stream ID for output frames.
    pub stream_id: u32,
}

/// How to handle complex numbers in MAT data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexMode {
    /// Use only the real part.
    RealOnly,

    /// Convert to magnitude (absolute value).
    Magnitude,

    /// Convert to magnitude and phase (adds columns).
    MagnitudePhase,

    /// Keep real and imaginary as separate columns.
    RealImag,
}

impl Default for ComplexMode {
    fn default() -> Self {
        ComplexMode::Magnitude
    }
}

impl Default for MatToSdifConfig {
    fn default() -> Self {
        MatToSdifConfig {
            time_variable: None,
            data_variable: None,
            frame_type: "1TRC".to_string(),
            matrix_type: "1TRC".to_string(),
            columns: vec![
                "Index".to_string(),
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "Phase".to_string(),
            ],
            max_partials: Some(1024),
            transpose: false,
            complex_mode: ComplexMode::default(),
            stream_id: 0,
        }
    }
}

impl MatToSdifConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the time variable name.
    pub fn time_var(mut self, name: impl Into<String>) -> Self {
        self.time_variable = Some(name.into());
        self
    }

    /// Set the data variable name.
    pub fn data_var(mut self, name: impl Into<String>) -> Self {
        self.data_variable = Some(name.into());
        self
    }

    /// Set the SDIF frame type.
    pub fn frame_type(mut self, sig: impl Into<String>) -> Self {
        self.frame_type = sig.into();
        self
    }

    /// Set the SDIF matrix type.
    pub fn matrix_type(mut self, sig: impl Into<String>) -> Self {
        self.matrix_type = sig.into();
        self
    }

    /// Set the column names.
    pub fn columns(mut self, names: &[&str]) -> Self {
        self.columns = names.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set maximum partials per frame.
    pub fn max_partials(mut self, max: usize) -> Self {
        self.max_partials = Some(max);
        self
    }

    /// Disable partial limiting.
    pub fn no_partial_limit(mut self) -> Self {
        self.max_partials = None;
        self
    }

    /// Set whether to transpose the data.
    pub fn transpose(mut self, t: bool) -> Self {
        self.transpose = t;
        self
    }

    /// Set how to handle complex data.
    pub fn complex_mode(mut self, mode: ComplexMode) -> Self {
        self.complex_mode = mode;
        self
    }

    /// Set the stream ID.
    pub fn stream_id(mut self, id: u32) -> Self {
        self.stream_id = id;
        self
    }
}

/// Converter for MAT to SDIF conversion.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter, SdifFile};
///
/// let mat = MatFile::open("analysis.mat")?;
/// let config = MatToSdifConfig::new()
///     .time_var("time")
///     .data_var("partials");
///
/// let converter = MatToSdifConverter::new(&mat, config)?;
///
/// // Get info about the conversion
/// println!("Will convert {} frames", converter.num_frames());
///
/// // Perform the conversion
/// let mut writer = SdifFile::builder()
///     .create("output.sdif")?
///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
///     .build()?;
///
/// converter.write_to(&mut writer)?;
/// writer.close()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct MatToSdifConverter<'a> {
    /// Configuration.
    config: MatToSdifConfig,

    /// Time values for each frame.
    times: Array1<f64>,

    /// Data array (rows = time frames, cols = data).
    data: Array2<f64>,

    /// Reference to source MatFile (for metadata).
    _source: &'a MatFile,
}

impl<'a> MatToSdifConverter<'a> {
    /// Create a new converter.
    ///
    /// # Arguments
    ///
    /// * `mat` - The loaded MAT file.
    /// * `config` - Conversion configuration.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidFormat`] if required variables are missing
    /// - [`Error::InvalidFormat`] if data shapes are incompatible
    pub fn new(mat: &'a MatFile, config: MatToSdifConfig) -> Result<Self> {
        // Find time variable
        let time_var = Self::find_time_variable(mat, &config)?;
        let times = time_var.to_array1()?;

        // Find data variable
        let data_var = Self::find_data_variable(mat, &config)?;
        let mut data = data_var.to_array2()?;

        // Handle complex data
        if data_var.is_complex() {
            data = Self::handle_complex(data_var, &config)?;
        }

        // Transpose if requested
        if config.transpose {
            data = data.t().to_owned();
        }

        // Validate dimensions
        let num_frames = times.len();
        let (data_rows, _data_cols) = data.dim();

        if data_rows != num_frames {
            return Err(Error::invalid_format(format!(
                "Time vector length ({}) doesn't match data rows ({}). \
                 Try setting transpose=true if data is column-per-frame.",
                num_frames, data_rows
            )));
        }

        Ok(MatToSdifConverter {
            config,
            times,
            data,
            _source: mat,
        })
    }

    /// Find the time variable.
    fn find_time_variable<'m>(
        mat: &'m MatFile,
        config: &MatToSdifConfig,
    ) -> Result<&'m MatData> {
        if let Some(ref name) = config.time_variable {
            mat.require(name)
        } else {
            // Auto-detect
            let time_vars = mat.find_time_vectors();
            match time_vars.len() {
                0 => Err(Error::invalid_format(
                    "No time vector found. Specify --time-var explicitly.",
                )),
                1 => mat.require(time_vars[0]),
                _ => Err(Error::invalid_format(format!(
                    "Multiple possible time vectors found: {:?}. Specify --time-var explicitly.",
                    time_vars
                ))),
            }
        }
    }

    /// Find the data variable.
    fn find_data_variable<'m>(
        mat: &'m MatFile,
        config: &MatToSdifConfig,
    ) -> Result<&'m MatData> {
        if let Some(ref name) = config.data_variable {
            mat.require(name)
        } else {
            // Find non-time 2D variables
            let candidates: Vec<_> = mat
                .iter()
                .filter(|(_, v)| v.is_2d() && !v.is_likely_time_vector())
                .map(|(n, _)| n)
                .collect();

            match candidates.len() {
                0 => Err(Error::invalid_format(
                    "No suitable data variable found. Specify --data-var explicitly.",
                )),
                1 => mat.require(candidates[0]),
                _ => Err(Error::invalid_format(format!(
                    "Multiple possible data variables found: {:?}. Specify --data-var explicitly.",
                    candidates
                ))),
            }
        }
    }

    /// Handle complex data according to configuration.
    fn handle_complex(data_var: &MatData, config: &MatToSdifConfig) -> Result<Array2<f64>> {
        match config.complex_mode {
            ComplexMode::RealOnly => data_var.to_array2(),
            ComplexMode::Magnitude => data_var.magnitude(),
            ComplexMode::MagnitudePhase => {
                let mag = data_var.magnitude()?;
                let phase = data_var.phase()?;
                // Concatenate magnitude and phase columns
                let (rows, cols) = mag.dim();
                let mut combined = Array2::zeros((rows, cols * 2));
                combined.slice_mut(ndarray::s![.., ..cols]).assign(&mag);
                combined
                    .slice_mut(ndarray::s![.., cols..])
                    .assign(&phase);
                Ok(combined)
            }
            ComplexMode::RealImag => {
                let real = data_var.to_array2()?;
                let imag = data_var.imag_to_array2()?;
                let (rows, cols) = real.dim();
                let mut combined = Array2::zeros((rows, cols * 2));
                combined.slice_mut(ndarray::s![.., ..cols]).assign(&real);
                combined.slice_mut(ndarray::s![.., cols..]).assign(&imag);
                Ok(combined)
            }
        }
    }

    /// Get the number of frames that will be written.
    pub fn num_frames(&self) -> usize {
        self.times.len()
    }

    /// Get the time range.
    pub fn time_range(&self) -> (f64, f64) {
        let min = self
            .times
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = self
            .times
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    /// Get the number of columns per frame.
    pub fn cols_per_frame(&self) -> usize {
        self.data.ncols()
    }

    /// Write all frames to an SDIF writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The SDIF writer to write frames to.
    ///
    /// # Errors
    ///
    /// Returns any errors from the underlying writer.
    pub fn write_to(&self, writer: &mut SdifWriter) -> Result<()> {
        let max_partials = self.config.max_partials.unwrap_or(usize::MAX);

        for (i, &time) in self.times.iter().enumerate() {
            let row = self.data.row(i);
            let row_data: Vec<f64> = row.iter().copied().collect();

            // Calculate number of partials (rows in SDIF matrix)
            let cols = self.config.columns.len();
            let num_values = row_data.len();

            if num_values % cols != 0 {
                return Err(Error::invalid_format(format!(
                    "Data length {} is not divisible by column count {}",
                    num_values, cols
                )));
            }

            let num_partials = (num_values / cols).min(max_partials);
            let limited_data = &row_data[..num_partials * cols];

            writer.write_frame_one_matrix(
                &self.config.frame_type,
                time,
                &self.config.matrix_type,
                num_partials,
                cols,
                limited_data,
            )?;
        }

        Ok(())
    }

    /// Get frame data for a specific time index.
    pub fn frame_data(&self, index: usize) -> Option<(&f64, ndarray::ArrayView1<f64>)> {
        if index < self.times.len() {
            Some((&self.times[index], self.data.row(index)))
        } else {
            None
        }
    }

    /// Iterate over (time, data) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (f64, ndarray::ArrayView1<f64>)> + '_ {
        self.times
            .iter()
            .copied()
            .zip(self.data.rows().into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = MatToSdifConfig::new()
            .frame_type("1HRM")
            .matrix_type("1HRM")
            .columns(&["Freq", "Amp"])
            .max_partials(512);

        assert_eq!(config.frame_type, "1HRM");
        assert_eq!(config.matrix_type, "1HRM");
        assert_eq!(config.columns, vec!["Freq", "Amp"]);
        assert_eq!(config.max_partials, Some(512));
    }
}
