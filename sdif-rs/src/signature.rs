//! SDIF signature (4-character code) utilities.
//!
//! SDIF uses 4-character ASCII codes to identify frame and matrix types.
//! These are stored as 32-bit unsigned integers for efficiency.
//!
//! Common signatures include:
//! - `1TRC` - Sinusoidal tracks
//! - `1HRM` - Harmonic partials
//! - `1FQ0` - Fundamental frequency
//!
//! # Example
//!
//! ```
//! use sdif_rs::{string_to_signature, signature_to_string};
//!
//! let sig = string_to_signature("1TRC").unwrap();
//! assert_eq!(signature_to_string(sig), "1TRC");
//! ```

use crate::error::{Error, Result};

/// A 4-character SDIF signature stored as a 32-bit integer.
pub type Signature = u32;

/// Convert a 4-character string to an SDIF signature.
///
/// # Arguments
///
/// * `s` - A string that must be exactly 4 ASCII characters.
///
/// # Returns
///
/// The signature as a `u32`, or an error if the string is invalid.
///
/// # Errors
///
/// Returns [`Error::InvalidSignature`] if:
/// - The string is not exactly 4 bytes
/// - The string contains non-ASCII characters
///
/// # Example
///
/// ```
/// use sdif_rs::string_to_signature;
///
/// let sig = string_to_signature("1TRC").unwrap();
/// assert_eq!(sig, 0x31545243); // '1' 'T' 'R' 'C' in big-endian
/// ```
pub fn string_to_signature(s: &str) -> Result<Signature> {
    let bytes = s.as_bytes();

    if bytes.len() != 4 {
        return Err(Error::invalid_signature(s));
    }

    // Verify all ASCII
    if !bytes.iter().all(|b| b.is_ascii()) {
        return Err(Error::invalid_signature(s));
    }

    Ok(sig_const_from_slice(bytes))
}

/// Convert an SDIF signature to its 4-character string representation.
///
/// # Arguments
///
/// * `sig` - The signature as a `u32`.
///
/// # Returns
///
/// A 4-character string. Non-printable bytes are replaced with '?'.
///
/// # Example
///
/// ```
/// use sdif_rs::signature_to_string;
///
/// let s = signature_to_string(0x31545243);
/// assert_eq!(s, "1TRC");
/// ```
pub fn signature_to_string(sig: Signature) -> String {
    let bytes = [
        ((sig >> 24) & 0xFF) as u8,
        ((sig >> 16) & 0xFF) as u8,
        ((sig >> 8) & 0xFF) as u8,
        (sig & 0xFF) as u8,
    ];

    // Replace non-printable with '?'
    let clean: Vec<u8> = bytes
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b } else { b'?' })
        .collect();

    String::from_utf8_lossy(&clean).into_owned()
}

/// Create a signature from a 4-byte array at compile time.
///
/// This is used internally to define signature constants.
#[doc(hidden)]
pub const fn sig_const(s: &[u8; 4]) -> Signature {
    ((s[0] as u32) << 24)
        | ((s[1] as u32) << 16)
        | ((s[2] as u32) << 8)
        | (s[3] as u32)
}

/// Create a signature from a byte slice (runtime version).
fn sig_const_from_slice(s: &[u8]) -> Signature {
    debug_assert_eq!(s.len(), 4);
    ((s[0] as u32) << 24)
        | ((s[1] as u32) << 16)
        | ((s[2] as u32) << 8)
        | (s[3] as u32)
}

/// Check if a signature matches a known type.
pub fn is_known_signature(sig: Signature) -> bool {
    matches!(
        sig,
        crate::signatures::TRC
            | crate::signatures::HRM
            | crate::signatures::FQ0
            | crate::signatures::RES
            | crate::signatures::STF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_signature() {
        let sig = string_to_signature("1TRC").unwrap();
        assert_eq!(sig, 0x31545243);

        let sig = string_to_signature("1HRM").unwrap();
        assert_eq!(sig, 0x3148524D);
    }

    #[test]
    fn test_signature_to_string() {
        assert_eq!(signature_to_string(0x31545243), "1TRC");
        assert_eq!(signature_to_string(0x3148524D), "1HRM");
    }

    #[test]
    fn test_roundtrip() {
        let original = "TEST";
        let sig = string_to_signature(original).unwrap();
        let recovered = signature_to_string(sig);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_invalid_signatures() {
        // Too short
        assert!(string_to_signature("ABC").is_err());

        // Too long
        assert!(string_to_signature("ABCDE").is_err());

        // Empty
        assert!(string_to_signature("").is_err());
    }

    #[test]
    fn test_const_signature() {
        assert_eq!(sig_const(b"1TRC"), 0x31545243);
    }

    #[test]
    fn test_known_signatures() {
        assert!(is_known_signature(crate::signatures::TRC));
        assert!(is_known_signature(crate::signatures::HRM));
        assert!(!is_known_signature(0x00000000));
    }
}
