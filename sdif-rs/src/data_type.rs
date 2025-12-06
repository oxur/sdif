//! SDIF data type enumeration.
//!
//! SDIF matrices can contain data in several numeric formats.
//! The most common are `Float4` (f32) and `Float8` (f64).

use std::fmt;

/// SDIF matrix data types.
///
/// SDIF supports various numeric data types for matrix storage.
/// In practice, most audio analysis data uses `Float4` or `Float8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DataType {
    /// 32-bit floating point (f32)
    Float4 = 0x0004,

    /// 64-bit floating point (f64)
    Float8 = 0x0008,

    /// 8-bit signed integer (i8)
    Int1 = 0x0101,

    /// 16-bit signed integer (i16)
    Int2 = 0x0102,

    /// 32-bit signed integer (i32)
    Int4 = 0x0104,

    /// 8-bit unsigned integer (u8)
    UInt1 = 0x0201,

    /// 16-bit unsigned integer (u16)
    UInt2 = 0x0202,

    /// 32-bit unsigned integer (u32)
    UInt4 = 0x0204,

    /// UTF-8 text data
    Text = 0x0301,

    /// Unknown or unsupported type
    Unknown = 0x0000,
}

impl DataType {
    /// Create a DataType from its raw C enum value.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw value from the C library.
    ///
    /// # Returns
    ///
    /// The corresponding `DataType`, or `Unknown` if not recognized.
    pub fn from_raw(value: u32) -> Self {
        match value {
            0x0004 => DataType::Float4,
            0x0008 => DataType::Float8,
            0x0101 => DataType::Int1,
            0x0102 => DataType::Int2,
            0x0104 => DataType::Int4,
            0x0201 => DataType::UInt1,
            0x0202 => DataType::UInt2,
            0x0204 => DataType::UInt4,
            0x0301 => DataType::Text,
            _ => DataType::Unknown,
        }
    }

    /// Get the size in bytes of a single element of this type.
    ///
    /// # Returns
    ///
    /// The byte size, or 0 for `Text` and `Unknown`.
    pub const fn size_bytes(&self) -> usize {
        match self {
            DataType::Float4 => 4,
            DataType::Float8 => 8,
            DataType::Int1 | DataType::UInt1 => 1,
            DataType::Int2 | DataType::UInt2 => 2,
            DataType::Int4 | DataType::UInt4 => 4,
            DataType::Text | DataType::Unknown => 0,
        }
    }

    /// Check if this type is a floating-point type.
    pub const fn is_float(&self) -> bool {
        matches!(self, DataType::Float4 | DataType::Float8)
    }

    /// Check if this type is an integer type.
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            DataType::Int1
                | DataType::Int2
                | DataType::Int4
                | DataType::UInt1
                | DataType::UInt2
                | DataType::UInt4
        )
    }

    /// Check if this type is a signed integer type.
    pub const fn is_signed(&self) -> bool {
        matches!(self, DataType::Int1 | DataType::Int2 | DataType::Int4)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Float4 => write!(f, "float32"),
            DataType::Float8 => write!(f, "float64"),
            DataType::Int1 => write!(f, "int8"),
            DataType::Int2 => write!(f, "int16"),
            DataType::Int4 => write!(f, "int32"),
            DataType::UInt1 => write!(f, "uint8"),
            DataType::UInt2 => write!(f, "uint16"),
            DataType::UInt4 => write!(f, "uint32"),
            DataType::Text => write!(f, "text"),
            DataType::Unknown => write!(f, "unknown"),
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Float8 // Most common for audio data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw() {
        assert_eq!(DataType::from_raw(0x0004), DataType::Float4);
        assert_eq!(DataType::from_raw(0x0008), DataType::Float8);
        assert_eq!(DataType::from_raw(0xFFFF), DataType::Unknown);
    }

    #[test]
    fn test_size_bytes() {
        assert_eq!(DataType::Float4.size_bytes(), 4);
        assert_eq!(DataType::Float8.size_bytes(), 8);
        assert_eq!(DataType::Int2.size_bytes(), 2);
    }

    #[test]
    fn test_type_checks() {
        assert!(DataType::Float4.is_float());
        assert!(DataType::Float8.is_float());
        assert!(!DataType::Int4.is_float());

        assert!(DataType::Int4.is_integer());
        assert!(DataType::Int4.is_signed());
        assert!(!DataType::UInt4.is_signed());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataType::Float4), "float32");
        assert_eq!(format!("{}", DataType::Float8), "float64");
    }
}
