//! Global SDIF library initialization.
//!
//! The SDIF C library requires initialization before any operations can be
//! performed. This module ensures the library is initialized exactly once,
//! in a thread-safe manner.
//!
//! Users don't need to call these functions directly - initialization is
//! handled automatically when opening an SDIF file.

use std::ptr;
use std::sync::Once;

use sdif_sys::SdifGenInit;

/// Static guard for one-time initialization.
static INIT: Once = Once::new();

/// Flag to track if initialization succeeded.
///
/// We use a simple atomic bool pattern here. In practice, SdifGenInit
/// always succeeds, but we track it for safety.
static mut INIT_SUCCEEDED: bool = false;

/// Ensures the SDIF library is initialized.
///
/// This function is safe to call multiple times from any thread - the
/// initialization will only happen once. Subsequent calls are no-ops.
///
/// # Returns
///
/// `true` if the library is (now) initialized, `false` if initialization failed.
///
/// # Example
///
/// ```
/// # use sdif_rs::init::ensure_initialized;
/// // Called automatically by SdifFile::open, but can be called manually
/// assert!(ensure_initialized());
/// ```
pub fn ensure_initialized() -> bool {
    INIT.call_once(|| {
        // SAFETY: SdifGenInit is called exactly once, protected by Once.
        // Passing null uses the default types file path.
        unsafe {
            SdifGenInit(ptr::null());
            INIT_SUCCEEDED = true;
        }
    });

    // SAFETY: INIT_SUCCEEDED is only written inside call_once,
    // which guarantees it completes before any read.
    unsafe { INIT_SUCCEEDED }
}

/// Check if the library has been initialized.
///
/// Returns `true` if `ensure_initialized()` has been called successfully.
pub fn is_initialized() -> bool {
    if INIT.is_completed() {
        // SAFETY: If Once is completed, INIT_SUCCEEDED has its final value.
        unsafe { INIT_SUCCEEDED }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(sdif_stub_bindings))]
    fn test_initialization() {
        // First call should initialize
        assert!(ensure_initialized());

        // Subsequent calls should be no-ops but still return true
        assert!(ensure_initialized());
        assert!(ensure_initialized());

        // Should report as initialized
        assert!(is_initialized());
    }
}
