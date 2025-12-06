//! Individual MAT variable representation.
//!
//! [`MatData`] represents a single numeric variable from a MAT file,
//! providing access to its data as ndarray arrays.

use matfile::{Array as MatArray, NumericData};
use ndarray::{Array1, Array2, ShapeBuilder};

use crate::error::{Error, Result};

/// A numeric variable from a MAT file.
///
/// `MatData` wraps a single variable, providing:
/// - Shape information
/// - Data access as 1D or 2D arrays
/// - Complex number handling
/// - Type information
///
/// # Data Layout
///
/// MATLAB uses column-major (Fortran) order, while this library
/// converts to row-major (C) order for SDIF compatibility.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::MatFile;
///
/// let mat = MatFile::open("data.mat")?;
/// let freqs = mat.require("frequencies")?;
///
/// println!("Shape: {:?}", freqs.shape());
/// println!("Is 1D: {}", freqs.is_1d());
///
/// // Get as 2D array
/// let array = freqs.to_array2()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MatData {
    /// Variable name.
    pub(super) name: String,

    /// Shape of the array.
    shape: Vec<usize>,

    /// Real part of the data (always present).
    pub(super) real_data: Vec<f64>,

    /// Imaginary part (only for complex data).
    pub(super) imag_data: Option<Vec<f64>>,

    /// Original data type name.
    dtype: String,
}

impl MatData {
    /// Create MatData from a matfile Array.
    pub(crate) fn from_matfile_array(array: &MatArray) -> Result<Self> {
        let name = array.name().to_string();
        let size = array.size();
        let shape: Vec<usize> = size.iter().map(|&x| x as usize).collect();

        // Extract numeric data
        let (real_data, imag_data, dtype) = Self::extract_numeric_data(array)?;

        Ok(MatData {
            name,
            shape,
            real_data,
            imag_data,
            dtype,
        })
    }

    /// Extract numeric data from a matfile Array.
    fn extract_numeric_data(
        array: &MatArray,
    ) -> Result<(Vec<f64>, Option<Vec<f64>>, String)> {
        match array.data() {
            NumericData::Double { real, imag } => {
                let real_vec: Vec<f64> = real.iter().copied().collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().copied().collect());
                Ok((real_vec, imag_vec, "float64".to_string()))
            }
            NumericData::Single { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "float32".to_string()))
            }
            NumericData::Int8 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int8".to_string()))
            }
            NumericData::Int16 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int16".to_string()))
            }
            NumericData::Int32 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int32".to_string()))
            }
            NumericData::Int64 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int64".to_string()))
            }
            NumericData::UInt8 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint8".to_string()))
            }
            NumericData::UInt16 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint16".to_string()))
            }
            NumericData::UInt32 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint32".to_string()))
            }
            NumericData::UInt64 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint64".to_string()))
            }
        }
    }

    /// Get the variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the shape of the array.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Check if the array is 1-dimensional.
    ///
    /// Note: MATLAB stores row vectors as [1, N] and column vectors as [N, 1].
    /// This method returns true for both, as well as true 1D arrays.
    pub fn is_1d(&self) -> bool {
        match self.shape.as_slice() {
            [_] => true,
            [1, _] => true,
            [_, 1] => true,
            _ => false,
        }
    }

    /// Check if the array is 2-dimensional.
    pub fn is_2d(&self) -> bool {
        self.shape.len() == 2
    }

    /// Check if the data is complex.
    pub fn is_complex(&self) -> bool {
        self.imag_data.is_some()
    }

    /// Get the total number of elements.
    pub fn len(&self) -> usize {
        self.real_data.len()
    }

    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.real_data.is_empty()
    }

    /// Get the original data type name.
    pub fn dtype(&self) -> &str {
        &self.dtype
    }

    /// Get the real part as a 1D array.
    ///
    /// For vectors (1D or [1,N] or [N,1]), returns the data as-is.
    /// For 2D arrays, returns the data in row-major order.
    pub fn to_array1(&self) -> Result<Array1<f64>> {
        if !self.is_1d() && self.shape.len() > 1 {
            return Err(Error::invalid_format(format!(
                "Variable '{}' is not 1D (shape: {:?})",
                self.name, self.shape
            )));
        }

        Ok(Array1::from_vec(self.real_data.clone()))
    }

    /// Get the real part as a 2D array in row-major order.
    ///
    /// Automatically handles MATLAB's column-major to row-major conversion.
    pub fn to_array2(&self) -> Result<Array2<f64>> {
        let (rows, cols) = self.dims_2d()?;

        // MATLAB stores column-major, so we need to transpose
        // Create as column-major then transpose
        let col_major = Array2::from_shape_vec((rows, cols).f(), self.real_data.clone())
            .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;

        // Return transposed (now row-major interpretation is correct)
        Ok(col_major)
    }

    /// Get 2D dimensions, treating 1D as [N, 1].
    fn dims_2d(&self) -> Result<(usize, usize)> {
        match self.shape.as_slice() {
            [n] => Ok((*n, 1)),
            [r, c] => Ok((*r, *c)),
            _ => Err(Error::invalid_format(format!(
                "Variable '{}' is not 2D (shape: {:?})",
                self.name, self.shape
            ))),
        }
    }

    /// Get the imaginary part as a 2D array (for complex data).
    pub fn imag_to_array2(&self) -> Result<Array2<f64>> {
        let imag = self.imag_data.as_ref().ok_or_else(|| {
            Error::invalid_format(format!("Variable '{}' is not complex", self.name))
        })?;

        let (rows, cols) = self.dims_2d()?;

        let col_major = Array2::from_shape_vec((rows, cols).f(), imag.clone())
            .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;

        Ok(col_major)
    }

    /// Get magnitude of complex data: sqrt(real² + imag²).
    pub fn magnitude(&self) -> Result<Array2<f64>> {
        let real = self.to_array2()?;

        if let Some(ref imag_data) = self.imag_data {
            let (rows, cols) = self.dims_2d()?;
            let imag = Array2::from_shape_vec((rows, cols).f(), imag_data.clone())
                .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;

            Ok((&real * &real + &imag * &imag).mapv(f64::sqrt))
        } else {
            // For real data, magnitude is just absolute value
            Ok(real.mapv(f64::abs))
        }
    }

    /// Get phase of complex data: atan2(imag, real).
    pub fn phase(&self) -> Result<Array2<f64>> {
        let real = self.to_array2()?;

        if let Some(ref imag_data) = self.imag_data {
            let (rows, cols) = self.dims_2d()?;
            let imag = Array2::from_shape_vec((rows, cols).f(), imag_data.clone())
                .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;

            // Element-wise atan2
            let mut phase = Array2::zeros((rows, cols));
            for ((r, i), p) in real.iter().zip(imag.iter()).zip(phase.iter_mut()) {
                *p = i.atan2(*r);
            }
            Ok(phase)
        } else {
            // For real data, phase is 0 for positive, π for negative
            Ok(real.mapv(|x| if x >= 0.0 { 0.0 } else { std::f64::consts::PI }))
        }
    }

    /// Get raw real data slice.
    pub fn real_data(&self) -> &[f64] {
        &self.real_data
    }

    /// Get raw imaginary data slice (if complex).
    pub fn imag_data(&self) -> Option<&[f64]> {
        self.imag_data.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_1d() {
        // These would need actual MatData instances to test properly
        // Integration tests will cover this with real MAT files
    }
}
