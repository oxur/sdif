//! SDIF file reading operations.
//!
//! This module provides [`SdifFile`], the main entry point for reading SDIF files.
//!
//! # Example
//!
//! ```no_run
//! use sdif_rs::SdifFile;
//!
//! let file = SdifFile::open("analysis.sdif")?;
//! println!("Opened SDIF file with {} NVT entries", file.nvts().len());
//! # Ok::<(), sdif_rs::Error>(())
//! ```

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use sdif_sys::{
    SdifFClose, SdifFOpen, SdifFReadAllASCIIChunks, SdifFReadGeneralHeader,
    SdifFileT, SdifFileModeET_eReadFile,
};

use crate::error::{Error, Result};
use crate::frame::FrameIterator;
use crate::init::ensure_initialized;

/// An SDIF file opened for reading.
///
/// `SdifFile` wraps the C library's file handle with RAII semantics.
/// The file is automatically closed when the `SdifFile` is dropped.
///
/// # Thread Safety
///
/// `SdifFile` is `!Send` and `!Sync` because the underlying C library
/// uses global state and is not thread-safe. All operations on an
/// `SdifFile` must occur on the same thread.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let file = SdifFile::open("input.sdif")?;
///
/// // Iterate over all frames
/// for frame_result in file.frames() {
///     let frame = frame_result?;
///     println!("Frame at time {:.3}s", frame.time());
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct SdifFile {
    /// Pointer to the C file handle. Never null after construction.
    handle: NonNull<SdifFileT>,

    /// Cached NVT (Name-Value Table) entries read from the file.
    nvts: Vec<HashMap<String, String>>,

    /// Track whether we're currently iterating frames.
    /// Prevents multiple simultaneous iterators.
    iterating: Cell<bool>,

    /// Marker to make SdifFile !Send and !Sync.
    /// The C library uses global state and isn't thread-safe.
    _not_send_sync: PhantomData<*const ()>,
}

impl SdifFile {
    /// Open an SDIF file for reading.
    ///
    /// This reads the general header and all ASCII chunks (NVT, type definitions).
    /// After opening, use [`frames()`](Self::frames) to iterate over data frames.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SDIF file.
    ///
    /// # Returns
    ///
    /// An `SdifFile` ready for reading, or an error if the file couldn't be opened.
    ///
    /// # Errors
    ///
    /// - [`Error::InitFailed`] if the SDIF library couldn't be initialized
    /// - [`Error::OpenFailed`] if the file doesn't exist or isn't readable
    /// - [`Error::InvalidFormat`] if the file isn't a valid SDIF file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("analysis.sdif")?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Ensure library is initialized
        if !ensure_initialized() {
            return Err(Error::InitFailed);
        }

        // Convert path to C string
        let path_str = path.to_str().ok_or_else(|| {
            Error::invalid_format("Path contains invalid UTF-8")
        })?;
        let c_path = CString::new(path_str)?;

        // Open the file
        let handle = unsafe {
            SdifFOpen(c_path.as_ptr(), SdifFileModeET_eReadFile)
        };

        let handle = NonNull::new(handle).ok_or_else(|| {
            Error::open_failed(path)
        })?;

        // Read general header
        let header_bytes = unsafe { SdifFReadGeneralHeader(handle.as_ptr()) };
        if header_bytes == 0 {
            // Clean up and return error
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::invalid_format("Failed to read SDIF header"));
        }

        // Read ASCII chunks (NVT, type definitions)
        let ascii_bytes = unsafe { SdifFReadAllASCIIChunks(handle.as_ptr()) };
        if ascii_bytes < 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::invalid_format("Failed to read ASCII chunks"));
        }

        // Parse NVTs
        let nvts = Self::read_nvts(handle.as_ptr());

        Ok(SdifFile {
            handle,
            nvts,
            iterating: Cell::new(false),
            _not_send_sync: PhantomData,
        })
    }

    /// Get the Name-Value Tables (NVT) from the file.
    ///
    /// NVTs contain metadata about the file, such as creator, date,
    /// sample rate, and other application-specific information.
    ///
    /// # Returns
    ///
    /// A slice of hash maps, where each map represents one NVT.
    /// Most files have a single NVT.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// for nvt in file.nvts() {
    ///     if let Some(creator) = nvt.get("creator") {
    ///         println!("Created by: {}", creator);
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn nvts(&self) -> &[HashMap<String, String>] {
        &self.nvts
    }

    /// Get a specific value from the first NVT.
    ///
    /// This is a convenience method for accessing common metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The NVT key to look up.
    ///
    /// # Returns
    ///
    /// The value if found, or `None`.
    pub fn nvt_get(&self, key: &str) -> Option<&str> {
        self.nvts.first()?.get(key).map(|s| s.as_str())
    }

    /// Create an iterator over all frames in the file.
    ///
    /// Frames are read sequentially from the current file position.
    /// Each frame contains one or more matrices of data.
    ///
    /// # Returns
    ///
    /// A [`FrameIterator`] that yields [`Frame`] objects.
    ///
    /// # Panics
    ///
    /// Panics if called while another frame iterator is active.
    /// Only one iterator can be active at a time because the
    /// underlying C library maintains file position state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// for frame_result in file.frames() {
    ///     let frame = frame_result?;
    ///     println!("Time: {:.3}, Sig: {}", frame.time(), frame.signature());
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn frames(&self) -> FrameIterator<'_> {
        if self.iterating.get() {
            panic!("Cannot create multiple frame iterators simultaneously");
        }
        self.iterating.set(true);
        FrameIterator::new(self)
    }

    /// Get the raw C file handle.
    ///
    /// # Safety
    ///
    /// This is for internal use. The caller must not close the handle
    /// or use it after the `SdifFile` is dropped.
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.handle.as_ptr()
    }

    /// Mark that frame iteration has ended.
    pub(crate) fn end_iteration(&self) {
        self.iterating.set(false);
    }

    /// Read NVT entries from the file.
    fn read_nvts(handle: *mut SdifFileT) -> Vec<HashMap<String, String>> {
        // TODO: Implement NVT reading using SDIF C API
        // For now, return empty vec - will implement with proper C API calls
        // The C API provides SdifFGetAllNVT, SdifNameValueTableGetNbData, etc.

        // Placeholder - actual implementation requires walking the NVT structures
        Vec::new()
    }
}

impl Drop for SdifFile {
    fn drop(&mut self) {
        // SAFETY: We own the handle and it's valid (NonNull).
        // After this, the handle is invalid, but we're being dropped anyway.
        unsafe {
            SdifFClose(self.handle.as_ptr());
        }
    }
}

// PhantomData<*const ()> makes SdifFile !Send and !Sync automatically

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_nonexistent() {
        let result = SdifFile::open("/nonexistent/path/to/file.sdif");
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::OpenFailed { path } => {
                assert!(path.to_str().unwrap().contains("nonexistent"));
            }
            other => panic!("Expected OpenFailed, got {:?}", other),
        }
    }

    // Additional tests require test fixtures - see integration tests
}
