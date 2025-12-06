//! SDIF matrix representation and data access.
//!
//! Matrices are the fundamental data containers in SDIF files.
//! Each matrix has a signature, dimensions (rows x columns), and
//! typed numeric data.

use std::marker::PhantomData;

use sdif_sys::{
    SdifFCurrDataType, SdifFCurrMatrixSignature, SdifFCurrNbCol,
    SdifFCurrNbRow, SdifFReadMatrixHeader,
    SdifFCurrOneRowData, SdifFReadOneRow, SdifFSkipMatrixData,
};

use crate::data_type::DataType;
use crate::error::{Error, Result};
use crate::frame::Frame;
use crate::signature::{signature_to_string, Signature};

#[cfg(feature = "ndarray")]
use ndarray::{Array2, ShapeBuilder};

/// A matrix of data from an SDIF frame.
///
/// Matrices contain 2D arrays of numeric data. Common columns include
/// Index, Frequency, Amplitude, and Phase for sinusoidal data.
///
/// # Data Access
///
/// Data can be accessed in several ways:
///
/// - [`data_f64()`](Self::data_f64) - Get all data as `Vec<f64>` (row-major)
/// - [`data_f32()`](Self::data_f32) - Get all data as `Vec<f32>` (row-major)
/// - [`to_array_f64()`](Self::to_array_f64) - Get as `ndarray::Array2<f64>` (requires `ndarray` feature)
///
/// # Example
///
/// ```no_run
/// # use sdif_rs::SdifFile;
/// let file = SdifFile::open("input.sdif")?;
/// for frame in file.frames() {
///     for matrix in frame?.matrices() {
///         let matrix = matrix?;
///         println!("Matrix '{}': {}x{} ({})",
///             matrix.signature(),
///             matrix.rows(),
///             matrix.cols(),
///             matrix.data_type()
///         );
///
///         // Get data as f64
///         let data = matrix.data_f64()?;
///         println!("Total elements: {}", data.len());
///     }
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct Matrix<'a> {
    /// Reference to parent frame (for file handle access).
    frame: &'a Frame<'a>,

    /// Matrix type signature.
    signature: Signature,

    /// Number of rows.
    rows: u32,

    /// Number of columns.
    cols: u32,

    /// Data type of matrix elements.
    data_type: DataType,

    /// Whether data has been read.
    data_read: bool,

    /// Lifetime marker.
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Matrix<'a> {
    /// Create a new Matrix from the current file state.
    ///
    /// This should only be called after SdifFReadMatrixHeader succeeds.
    pub(crate) fn from_current(frame: &'a Frame<'a>) -> Self {
        let handle = frame.handle();

        let signature = unsafe { SdifFCurrMatrixSignature(handle) };
        let rows = unsafe { SdifFCurrNbRow(handle) };
        let cols = unsafe { SdifFCurrNbCol(handle) };
        let raw_dtype = unsafe { SdifFCurrDataType(handle) };
        let data_type = DataType::from_raw(raw_dtype as u32);

        Matrix {
            frame,
            signature,
            rows,
            cols,
            data_type,
            data_read: false,
            _phantom: PhantomData,
        }
    }

    /// Get the matrix type signature as a string (e.g., "1TRC").
    pub fn signature(&self) -> String {
        signature_to_string(self.signature)
    }

    /// Get the matrix type signature as a raw u32.
    pub fn signature_raw(&self) -> Signature {
        self.signature
    }

    /// Get the number of rows in the matrix.
    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    /// Get the number of columns in the matrix.
    pub fn cols(&self) -> usize {
        self.cols as usize
    }

    /// Get the total number of elements in the matrix.
    pub fn len(&self) -> usize {
        self.rows() * self.cols()
    }

    /// Check if the matrix is empty (no elements).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the data type of matrix elements.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Get the matrix dimensions as a tuple (rows, cols).
    pub fn shape(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }

    /// Read matrix data as f64 values in row-major order.
    ///
    /// This reads all matrix data and converts to f64 if necessary.
    /// The data is returned in row-major order (C order).
    ///
    /// # Returns
    ///
    /// A vector of f64 values with length `rows * cols`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidState`] if data was already read
    /// - [`Error::ReadError`] if data couldn't be read
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let mut frame = file.frames().next().unwrap()?;
    /// # let matrix = frame.matrices().next().unwrap()?;
    /// let data = matrix.data_f64()?;
    ///
    /// // Access element at row 2, col 3
    /// let cols = matrix.cols();
    /// let value = data[2 * cols + 3];
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn data_f64(mut self) -> Result<Vec<f64>> {
        if self.data_read {
            return Err(Error::invalid_state("Matrix data already read"));
        }
        self.data_read = true;

        let handle = self.frame.handle();
        let total_elements = self.len();
        let mut data = Vec::with_capacity(total_elements);

        // Read row by row
        for _row in 0..self.rows {
            let bytes_read = unsafe { SdifFReadOneRow(handle) };
            if bytes_read <= 0 {
                return Err(Error::read_error("Failed to read matrix row"));
            }

            // Get pointer to row data
            let row_data = unsafe { SdifFCurrOneRowData(handle) };
            if row_data.is_null() {
                return Err(Error::null_pointer("Row data pointer"));
            }

            // Copy data based on type
            match self.data_type {
                DataType::Float8 => {
                    let ptr = row_data as *const f64;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) });
                    }
                }
                DataType::Float4 => {
                    let ptr = row_data as *const f32;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) } as f64);
                    }
                }
                _ => {
                    return Err(Error::type_mismatch("float", self.data_type.to_string()));
                }
            }
        }

        Ok(data)
    }

    /// Read matrix data as f32 values in row-major order.
    ///
    /// Similar to [`data_f64()`](Self::data_f64) but returns f32 values.
    /// If the source data is f64, it will be truncated to f32.
    pub fn data_f32(mut self) -> Result<Vec<f32>> {
        if self.data_read {
            return Err(Error::invalid_state("Matrix data already read"));
        }
        self.data_read = true;

        let handle = self.frame.handle();
        let total_elements = self.len();
        let mut data = Vec::with_capacity(total_elements);

        for _row in 0..self.rows {
            let bytes_read = unsafe { SdifFReadOneRow(handle) };
            if bytes_read <= 0 {
                return Err(Error::read_error("Failed to read matrix row"));
            }

            let row_data = unsafe { SdifFCurrOneRowData(handle) };
            if row_data.is_null() {
                return Err(Error::null_pointer("Row data pointer"));
            }

            match self.data_type {
                DataType::Float4 => {
                    let ptr = row_data as *const f32;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) });
                    }
                }
                DataType::Float8 => {
                    let ptr = row_data as *const f64;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) } as f32);
                    }
                }
                _ => {
                    return Err(Error::type_mismatch("float", self.data_type.to_string()));
                }
            }
        }

        Ok(data)
    }

    /// Read matrix data as an ndarray Array2<f64>.
    ///
    /// Requires the `ndarray` feature.
    ///
    /// # Returns
    ///
    /// A 2D array with shape (rows, cols) in row-major (C) order.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "ndarray")]
    /// # fn example() -> sdif_rs::Result<()> {
    /// use sdif_rs::SdifFile;
    /// use ndarray::Array2;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// let mut frame = file.frames().next().unwrap()?;
    /// let matrix = frame.matrices().next().unwrap()?;
    ///
    /// let array: Array2<f64> = matrix.to_array_f64()?;
    /// println!("Shape: {:?}", array.shape());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ndarray")]
    pub fn to_array_f64(self) -> Result<Array2<f64>> {
        let shape = self.shape();
        let data = self.data_f64()?;

        // Create array in row-major (C) order
        Array2::from_shape_vec(shape.strides((shape.1, 1)), data)
            .map_err(|e| Error::invalid_format(format!("Array shape error: {}", e)))
    }

    /// Read matrix data as an ndarray Array2<f32>.
    ///
    /// Requires the `ndarray` feature.
    #[cfg(feature = "ndarray")]
    pub fn to_array_f32(self) -> Result<Array2<f32>> {
        let shape = self.shape();
        let data = self.data_f32()?;

        Array2::from_shape_vec(shape.strides((shape.1, 1)), data)
            .map_err(|e| Error::invalid_format(format!("Array shape error: {}", e)))
    }

    /// Skip this matrix's data without reading it.
    ///
    /// Useful when you want to skip matrices you're not interested in.
    pub fn skip(mut self) -> Result<()> {
        if self.data_read {
            return Ok(());
        }
        self.data_read = true;

        let result = unsafe { SdifFSkipMatrixData(self.frame.handle()) };
        if result < 0 {
            Err(Error::read_error("Failed to skip matrix data"))
        } else {
            Ok(())
        }
    }
}

impl Drop for Matrix<'_> {
    fn drop(&mut self) {
        // If data wasn't read, skip it to maintain file position
        if !self.data_read {
            unsafe {
                SdifFSkipMatrixData(self.frame.handle());
            }
            self.data_read = true;
        }
    }
}

/// Iterator over matrices in a frame.
///
/// Created by [`Frame::matrices()`].
pub struct MatrixIterator<'f, 'a: 'f> {
    frame: &'f mut Frame<'a>,
}

impl<'f, 'a: 'f> MatrixIterator<'f, 'a> {
    pub(crate) fn new(frame: &'f mut Frame<'a>) -> Self {
        MatrixIterator { frame }
    }
}

impl<'f, 'a: 'f> Iterator for MatrixIterator<'f, 'a> {
    type Item = Result<Matrix<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.frame.has_more_matrices() {
            self.frame.mark_finished();
            return None;
        }

        let handle = self.frame.handle();

        // Read matrix header
        let bytes_read = unsafe { SdifFReadMatrixHeader(handle) };

        if bytes_read <= 0 {
            return Some(Err(Error::read_error("Failed to read matrix header")));
        }

        self.frame.advance_matrix();

        // SAFETY: We need to create a Matrix with the same lifetime as the frame.
        // This is safe because the iterator borrows frame mutably, preventing
        // other access, and Matrix will be dropped before the next iteration.
        let matrix = unsafe {
            std::mem::transmute::<Matrix<'_>, Matrix<'a>>(Matrix::from_current(self.frame))
        };

        Some(Ok(matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Float4.size_bytes(), 4);
        assert_eq!(DataType::Float8.size_bytes(), 8);
    }
}
