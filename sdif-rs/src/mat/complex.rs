//! Complex number handling for MAT arrays.
//!
//! MATLAB stores complex numbers as separate real and imaginary arrays.
//! This module provides utilities for converting to various representations.

use ndarray::Array2;

use crate::error::{Error, Result};

/// Convert complex data to magnitude.
///
/// magnitude = sqrt(real² + imag²)
pub fn to_magnitude(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    if real.dim() != imag.dim() {
        return Err(Error::invalid_format(format!(
            "Real and imaginary arrays have different shapes: {:?} vs {:?}",
            real.dim(),
            imag.dim()
        )));
    }

    Ok((real * real + imag * imag).mapv(f64::sqrt))
}

/// Convert complex data to phase.
///
/// phase = atan2(imag, real)
pub fn to_phase(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    if real.dim() != imag.dim() {
        return Err(Error::invalid_format(format!(
            "Real and imaginary arrays have different shapes: {:?} vs {:?}",
            real.dim(),
            imag.dim()
        )));
    }

    let (rows, cols) = real.dim();
    let mut phase = Array2::zeros((rows, cols));

    for ((r, i), p) in real.iter().zip(imag.iter()).zip(phase.iter_mut()) {
        *p = i.atan2(*r);
    }

    Ok(phase)
}

/// Convert complex data to dB magnitude.
///
/// db = 20 * log10(magnitude)
pub fn to_db(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    let mag = to_magnitude(real, imag)?;

    // Avoid log(0) by clamping to a small value
    let min_val = 1e-10;
    Ok(mag.mapv(|x| 20.0 * x.max(min_val).log10()))
}

/// Unwrap phase to remove discontinuities.
///
/// Adjusts phase values to avoid jumps greater than π.
pub fn unwrap_phase(phase: &Array2<f64>) -> Array2<f64> {
    let (rows, cols) = phase.dim();
    let mut unwrapped = phase.clone();

    let pi = std::f64::consts::PI;
    let two_pi = 2.0 * pi;

    // Unwrap along rows (time axis)
    for col in 0..cols {
        let mut cumulative_offset = 0.0;

        for row in 1..rows {
            let prev = unwrapped[[row - 1, col]];
            let curr = phase[[row, col]];
            let diff = curr - prev + cumulative_offset;

            // Check for discontinuity
            if diff > pi {
                cumulative_offset -= two_pi;
            } else if diff < -pi {
                cumulative_offset += two_pi;
            }

            unwrapped[[row, col]] = curr + cumulative_offset;
        }
    }

    unwrapped
}

/// Convert polar (magnitude, phase) to rectangular (real, imag).
pub fn polar_to_rectangular(
    mag: &Array2<f64>,
    phase: &Array2<f64>,
) -> Result<(Array2<f64>, Array2<f64>)> {
    if mag.dim() != phase.dim() {
        return Err(Error::invalid_format(format!(
            "Magnitude and phase arrays have different shapes: {:?} vs {:?}",
            mag.dim(),
            phase.dim()
        )));
    }

    let real = mag * &phase.mapv(f64::cos);
    let imag = mag * &phase.mapv(f64::sin);

    Ok((real, imag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::array;

    #[test]
    fn test_magnitude() {
        let real = array![[3.0, 0.0], [0.0, 1.0]];
        let imag = array![[4.0, 1.0], [1.0, 0.0]];

        let mag = to_magnitude(&real, &imag).unwrap();

        assert_relative_eq!(mag[[0, 0]], 5.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[0, 1]], 1.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[1, 0]], 1.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[1, 1]], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_phase() {
        let real = array![[1.0, -1.0], [0.0, 0.0]];
        let imag = array![[0.0, 0.0], [1.0, -1.0]];

        let phase = to_phase(&real, &imag).unwrap();

        assert_relative_eq!(phase[[0, 0]], 0.0, epsilon = 1e-10);
        assert_relative_eq!(phase[[0, 1]], std::f64::consts::PI, epsilon = 1e-10);
        assert_relative_eq!(phase[[1, 0]], std::f64::consts::FRAC_PI_2, epsilon = 1e-10);
        assert_relative_eq!(phase[[1, 1]], -std::f64::consts::FRAC_PI_2, epsilon = 1e-10);
    }
}
