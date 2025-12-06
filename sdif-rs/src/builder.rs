//! SDIF file builder with typestate pattern.
//!
//! The builder uses Rust's type system to enforce valid state transitions
//! at compile time. You cannot write frames before setting up the file,
//! and you cannot modify configuration after headers are written.
//!
//! # State Machine
//!
//! ```text
//! New → (create) → Config → (build) → SdifWriter
//! ```
//!
//! # Example
//!
//! ```no_run
//! use sdif_rs::SdifFile;
//!
//! let mut writer = SdifFile::builder()
//!     .create("output.sdif")?
//!     .add_nvt([("creator", "my-app"), ("date", "2024-01-01")])?
//!     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
//!     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
//!     .build()?;
//!
//! // Now we can write frames
//! # Ok::<(), sdif_rs::Error>(())
//! ```

use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use sdif_sys::{
    SdifFOpen, SdifFClose, SdifFileT, SdifFileModeET_eWriteFile,
    SdifFWriteGeneralHeader, SdifFWriteAllASCIIChunks,
};

use crate::error::{Error, Result};
use crate::init::ensure_initialized;
use crate::writer::SdifWriter;

// ============================================================================
// Typestate Marker Types
// ============================================================================

/// Marker type: Builder just created, no path set yet.
#[derive(Debug)]
pub struct New;

/// Marker type: Path set, can configure NVT and types.
#[derive(Debug)]
pub struct Config;

// ============================================================================
// Configuration Storage
// ============================================================================

/// Stores NVT (Name-Value Table) entries.
#[derive(Debug, Default, Clone)]
pub(crate) struct NvtConfig {
    /// List of NVT tables, each being a map of key-value pairs.
    pub tables: Vec<HashMap<String, String>>,
}

/// Stores a matrix type definition.
#[derive(Debug, Clone)]
pub(crate) struct MatrixTypeDef {
    /// Matrix signature.
    pub signature: String,
    /// Column names.
    pub column_names: Vec<String>,
}

/// Stores a frame type definition.
#[derive(Debug, Clone)]
pub(crate) struct FrameTypeDef {
    /// Frame signature.
    pub signature: String,
    /// Component definitions.
    pub components: Vec<String>,
}

/// All configuration collected during the builder phase.
#[derive(Debug, Default, Clone)]
pub(crate) struct BuilderConfig {
    /// NVT tables.
    pub nvts: NvtConfig,
    /// Matrix type definitions.
    pub matrix_types: Vec<MatrixTypeDef>,
    /// Frame type definitions.
    pub frame_types: Vec<FrameTypeDef>,
}

// ============================================================================
// SdifFileBuilder
// ============================================================================

/// Builder for creating new SDIF files.
///
/// Uses the typestate pattern to enforce valid state transitions at compile time.
///
/// # Type Parameters
///
/// * `State` - The current state of the builder (New, Config)
#[derive(Debug)]
pub struct SdifFileBuilder<State> {
    /// Path to the output file (set after create()).
    path: Option<PathBuf>,

    /// Configuration accumulated during setup.
    config: BuilderConfig,

    /// Phantom data for the state type.
    _state: PhantomData<State>,
}

impl SdifFileBuilder<New> {
    /// Create a new builder in the initial state.
    ///
    /// This is typically called via `SdifFile::builder()`.
    pub fn new() -> Self {
        SdifFileBuilder {
            path: None,
            config: BuilderConfig::default(),
            _state: PhantomData,
        }
    }

    /// Set the output path and transition to Config state.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the SDIF file will be created.
    ///
    /// # Returns
    ///
    /// Builder in `Config` state, ready for configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?;
    /// // builder is now SdifFileBuilder<Config>
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn create(self, path: impl AsRef<Path>) -> Result<SdifFileBuilder<Config>> {
        let path = path.as_ref().to_path_buf();

        // Validate path is writable (parent directory exists)
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Parent directory does not exist: {}", parent.display()),
                )));
            }
        }

        Ok(SdifFileBuilder {
            path: Some(path),
            config: self.config,
            _state: PhantomData,
        })
    }
}

impl Default for SdifFileBuilder<New> {
    fn default() -> Self {
        Self::new()
    }
}

impl SdifFileBuilder<Config> {
    /// Add a Name-Value Table (NVT) with metadata.
    ///
    /// NVTs store metadata like creator, date, sample rate, etc.
    /// You can call this multiple times to add multiple NVTs.
    ///
    /// # Arguments
    ///
    /// * `entries` - Key-value pairs to add to the NVT.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_nvt([
    ///         ("creator", "my-application"),
    ///         ("date", "2024-01-01"),
    ///         ("sample_rate", "44100"),
    ///     ])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_nvt<'a>(
        mut self,
        entries: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self> {
        let mut nvt = HashMap::new();
        for (key, value) in entries {
            // Validate no embedded nulls
            if key.contains('\0') || value.contains('\0') {
                return Err(Error::invalid_format("NVT key/value cannot contain null bytes"));
            }
            nvt.insert(key.to_string(), value.to_string());
        }

        if !nvt.is_empty() {
            self.config.nvts.tables.push(nvt);
        }

        Ok(self)
    }

    /// Define a matrix type with column names.
    ///
    /// Matrix types define the structure of data matrices. Common types include:
    /// - `1TRC` with columns `["Index", "Frequency", "Amplitude", "Phase"]`
    /// - `1FQ0` with columns `["Frequency", "Confidence"]`
    ///
    /// # Arguments
    ///
    /// * `signature` - 4-character signature (e.g., "1TRC")
    /// * `columns` - Names of the columns in the matrix
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_matrix_type("1FQ0", &["Frequency", "Confidence"])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_matrix_type(mut self, signature: &str, columns: &[&str]) -> Result<Self> {
        // Validate signature
        if signature.len() != 4 {
            return Err(Error::invalid_signature(signature));
        }

        // Validate columns
        if columns.is_empty() {
            return Err(Error::invalid_format("Matrix type must have at least one column"));
        }

        for col in columns {
            if col.contains('\0') || col.contains(',') {
                return Err(Error::invalid_format(
                    "Column names cannot contain null bytes or commas"
                ));
            }
        }

        self.config.matrix_types.push(MatrixTypeDef {
            signature: signature.to_string(),
            column_names: columns.iter().map(|s| s.to_string()).collect(),
        });

        Ok(self)
    }

    /// Define a frame type with its component matrices.
    ///
    /// Frame types define what matrices can appear in a frame.
    /// The components string format is `"MatrixSig MatrixName"`.
    ///
    /// # Arguments
    ///
    /// * `signature` - 4-character signature (e.g., "1TRC")
    /// * `components` - Component definitions (e.g., `["1TRC SinusoidalTracks"]`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_frame_type(mut self, signature: &str, components: &[&str]) -> Result<Self> {
        // Validate signature
        if signature.len() != 4 {
            return Err(Error::invalid_signature(signature));
        }

        if components.is_empty() {
            return Err(Error::invalid_format("Frame type must have at least one component"));
        }

        self.config.frame_types.push(FrameTypeDef {
            signature: signature.to_string(),
            components: components.iter().map(|s| s.to_string()).collect(),
        });

        Ok(self)
    }

    /// Finalize configuration and create the writer.
    ///
    /// This opens the file, writes the general header and ASCII chunks
    /// (NVT, type definitions), and returns an `SdifWriter` ready to
    /// write frames.
    ///
    /// # Returns
    ///
    /// An `SdifWriter` for writing frames to the file.
    ///
    /// # Errors
    ///
    /// - [`Error::InitFailed`] if the SDIF library couldn't be initialized
    /// - [`Error::OpenFailed`] if the file couldn't be created
    /// - [`Error::Io`] if writing headers fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let mut writer = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    ///     .build()?;
    ///
    /// // writer is ready to write frames
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<SdifWriter> {
        // Ensure library is initialized
        if !ensure_initialized() {
            return Err(Error::InitFailed);
        }

        let path = self.path.as_ref().expect("Path should be set in Config state");

        // Convert path to C string
        let path_str = path.to_str().ok_or_else(|| {
            Error::invalid_format("Path contains invalid UTF-8")
        })?;
        let c_path = CString::new(path_str)?;

        // Open file for writing
        let handle = unsafe {
            SdifFOpen(c_path.as_ptr(), SdifFileModeET_eWriteFile)
        };

        let handle = NonNull::new(handle).ok_or_else(|| {
            Error::open_failed(path)
        })?;

        // Write NVT and type definitions to the file
        Self::write_ascii_chunks(handle.as_ptr(), &self.config)?;

        // Write general header
        let header_bytes = unsafe { SdifFWriteGeneralHeader(handle.as_ptr()) };
        if header_bytes == 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write SDIF header",
            )));
        }

        // Write ASCII chunks (NVT, type definitions)
        let ascii_bytes = unsafe { SdifFWriteAllASCIIChunks(handle.as_ptr()) };
        if ascii_bytes < 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write ASCII chunks",
            )));
        }

        Ok(SdifWriter::new(handle, path.clone()))
    }

    /// Write NVT and type definitions to the file handle.
    ///
    /// This is called before SdifFWriteAllASCIIChunks to set up the
    /// internal structures that will be written.
    fn write_ascii_chunks(handle: *mut SdifFileT, config: &BuilderConfig) -> Result<()> {
        // Add NVT entries
        for nvt in &config.nvts.tables {
            Self::add_nvt_to_file(handle, nvt)?;
        }

        // Add matrix type definitions
        for mtd in &config.matrix_types {
            Self::add_matrix_type_to_file(handle, mtd)?;
        }

        // Add frame type definitions
        for ftd in &config.frame_types {
            Self::add_frame_type_to_file(handle, ftd)?;
        }

        Ok(())
    }

    /// Add a single NVT to the file.
    fn add_nvt_to_file(handle: *mut SdifFileT, nvt: &HashMap<String, String>) -> Result<()> {
        use sdif_sys::{SdifFNameValueList, SdifNameValuesLNewTable, SdifNameValuesLPutCurrNVT};

        unsafe {
            // Get the NVT list from the file
            let nvt_list = SdifFNameValueList(handle);
            if nvt_list.is_null() {
                return Err(Error::null_pointer("NVT list"));
            }

            // Create a new NVT
            let stream_id = 0u32; // Default stream
            SdifNameValuesLNewTable(nvt_list, stream_id);

            // Add each key-value pair
            for (key, value) in nvt {
                let c_key = CString::new(key.as_str())?;
                let c_value = CString::new(value.as_str())?;

                SdifNameValuesLPutCurrNVT(nvt_list, c_key.as_ptr(), c_value.as_ptr());
            }
        }

        Ok(())
    }

    /// Add a matrix type definition to the file.
    fn add_matrix_type_to_file(handle: *mut SdifFileT, mtd: &MatrixTypeDef) -> Result<()> {
        use sdif_sys::{
            SdifFGetMatrixTypesTable, SdifMatrixTypeInsertTailColumnDef,
            SdifCreateMatrixType, SdifPutMatrixType,
        };
        use crate::signature::string_to_signature;

        unsafe {
            // Get the matrix types table
            let mtypes = SdifFGetMatrixTypesTable(handle);
            if mtypes.is_null() {
                return Err(Error::null_pointer("Matrix types table"));
            }

            // Create signature
            let sig = string_to_signature(&mtd.signature)?;

            // Create the matrix type (pass null for predefined)
            let mtype = SdifCreateMatrixType(sig, std::ptr::null_mut());
            if mtype.is_null() {
                return Err(Error::null_pointer("Matrix type"));
            }

            // Add column definitions
            for col_name in &mtd.column_names {
                let c_name = CString::new(col_name.as_str())?;
                SdifMatrixTypeInsertTailColumnDef(mtype, c_name.as_ptr());
            }

            // Add the matrix type to the table
            SdifPutMatrixType(mtypes, mtype);
        }

        Ok(())
    }

    /// Add a frame type definition to the file.
    fn add_frame_type_to_file(handle: *mut SdifFileT, ftd: &FrameTypeDef) -> Result<()> {
        use sdif_sys::{
            SdifFGetFrameTypesTable, SdifFrameTypePutComponent,
            SdifCreateFrameType, SdifPutFrameType,
        };
        use crate::signature::string_to_signature;

        unsafe {
            // Get the frame types table
            let ftypes = SdifFGetFrameTypesTable(handle);
            if ftypes.is_null() {
                return Err(Error::null_pointer("Frame types table"));
            }

            // Create signature
            let sig = string_to_signature(&ftd.signature)?;

            // Create the frame type (pass null for predefined)
            let ftype = SdifCreateFrameType(sig, std::ptr::null_mut());
            if ftype.is_null() {
                return Err(Error::null_pointer("Frame type"));
            }

            // Add component definitions
            // Components are in format "MSIG ComponentName"
            for component in &ftd.components {
                let parts: Vec<&str> = component.splitn(2, ' ').collect();
                if parts.len() != 2 || parts[0].len() != 4 {
                    return Err(Error::invalid_format(format!(
                        "Invalid component format: '{}' (expected 'MSIG Name')",
                        component
                    )));
                }

                let msig = string_to_signature(parts[0])?;
                let mut c_name = CString::new(parts[1])?;
                SdifFrameTypePutComponent(ftype, msig, c_name.as_ptr() as *mut _);
            }

            // Add the frame type to the table
            SdifPutFrameType(ftypes, ftype);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = SdifFileBuilder::<New>::new();
        assert!(builder.path.is_none());
    }

    #[test]
    fn test_builder_create_transitions_state() {
        let builder = SdifFileBuilder::<New>::new();

        // This should compile and transition to Config state
        let _config_builder: SdifFileBuilder<Config> = builder
            .create("/tmp/test.sdif")
            .unwrap();
    }

    #[test]
    fn test_invalid_signature_length() {
        let builder = SdifFileBuilder::<New>::new()
            .create("/tmp/test.sdif")
            .unwrap();

        let result = builder.add_matrix_type("TOOLONG", &["Col1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_columns_rejected() {
        let builder = SdifFileBuilder::<New>::new()
            .create("/tmp/test.sdif")
            .unwrap();

        let result = builder.add_matrix_type("1TRC", &[]);
        assert!(result.is_err());
    }
}
