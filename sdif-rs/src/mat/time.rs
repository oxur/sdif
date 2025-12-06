//! Time vector detection heuristics.
//!
//! This module provides methods for identifying time vectors in MAT files,
//! which are needed to generate SDIF frame timestamps.

use super::data::MatData;

/// Common names for time variables in audio analysis MAT files.
const TIME_VARIABLE_NAMES: &[&str] = &[
    "time",
    "times",
    "t",
    "Time",
    "Times",
    "T",
    "time_vec",
    "time_vector",
    "timeVec",
    "timeVector",
    "frame_times",
    "frameTimes",
    "frame_time",
];

impl MatData {
    /// Check if this variable is likely a time vector.
    ///
    /// Uses multiple heuristics:
    /// 1. Name matches common time variable names
    /// 2. Array is 1D (or effectively 1D)
    /// 3. Values are monotonically increasing
    /// 4. Values start near zero (within 1 second)
    /// 5. Values are non-negative
    ///
    /// # Returns
    ///
    /// `true` if the variable appears to be a time vector.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// for (name, var) in mat.iter() {
    ///     if var.is_likely_time_vector() {
    ///         println!("{} looks like a time vector", name);
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn is_likely_time_vector(&self) -> bool {
        // Must be 1D
        if !self.is_1d() {
            return false;
        }

        // Must have data
        if self.is_empty() || self.len() < 2 {
            return false;
        }

        // Check name heuristic
        let name_matches = TIME_VARIABLE_NAMES
            .iter()
            .any(|&n| self.name.eq_ignore_ascii_case(n));

        // Check value patterns
        let values_look_like_time = self.check_time_value_patterns();

        // If name matches, be more lenient with values
        // If name doesn't match, require stronger value evidence
        if name_matches {
            values_look_like_time || self.is_monotonically_increasing()
        } else {
            values_look_like_time && self.is_monotonically_increasing()
        }
    }

    /// Check if values follow typical time vector patterns.
    fn check_time_value_patterns(&self) -> bool {
        let data = &self.real_data;

        if data.is_empty() {
            return false;
        }

        // Check first value is near zero (within 10 seconds, typical for audio)
        let first = data[0];
        if first < -0.001 || first > 10.0 {
            return false;
        }

        // Check all values are non-negative
        if data.iter().any(|&x| x < -0.001) {
            return false;
        }

        // Check monotonically increasing
        self.is_monotonically_increasing()
    }

    /// Check if values are monotonically increasing.
    pub fn is_monotonically_increasing(&self) -> bool {
        let data = &self.real_data;

        if data.len() < 2 {
            return true;
        }

        // Allow small epsilon for floating point comparison
        let eps = 1e-10;

        for window in data.windows(2) {
            if window[1] < window[0] - eps {
                return false;
            }
        }

        true
    }

    /// Check if values are strictly increasing (no duplicates).
    pub fn is_strictly_increasing(&self) -> bool {
        let data = &self.real_data;

        if data.len() < 2 {
            return true;
        }

        let eps = 1e-10;

        for window in data.windows(2) {
            if window[1] <= window[0] + eps {
                return false;
            }
        }

        true
    }

    /// Get time statistics (useful for validation).
    ///
    /// Returns (min, max, mean_delta) if this is a valid time vector.
    pub fn time_stats(&self) -> Option<TimeStats> {
        if !self.is_1d() || self.len() < 2 {
            return None;
        }

        let data = &self.real_data;
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Calculate mean delta (hop size)
        let deltas: Vec<f64> = data.windows(2).map(|w| w[1] - w[0]).collect();
        let mean_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;

        // Calculate delta variance (for regularity check)
        let delta_variance = deltas
            .iter()
            .map(|d| (d - mean_delta).powi(2))
            .sum::<f64>()
            / deltas.len() as f64;

        Some(TimeStats {
            min,
            max,
            duration: max - min,
            num_frames: data.len(),
            mean_hop: mean_delta,
            hop_variance: delta_variance,
            is_regular: delta_variance < mean_delta * 0.01, // <1% variance
        })
    }
}

/// Statistics about a time vector.
#[derive(Debug, Clone)]
pub struct TimeStats {
    /// Minimum time value.
    pub min: f64,

    /// Maximum time value.
    pub max: f64,

    /// Total duration (max - min).
    pub duration: f64,

    /// Number of time points.
    pub num_frames: usize,

    /// Mean hop size between frames.
    pub mean_hop: f64,

    /// Variance in hop size.
    pub hop_variance: f64,

    /// Whether the hop size is regular (consistent spacing).
    pub is_regular: bool,
}

impl std::fmt::Display for TimeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TimeStats {{ duration: {:.3}s, frames: {}, hop: {:.4}s{} }}",
            self.duration,
            self.num_frames,
            self.mean_hop,
            if self.is_regular {
                " (regular)"
            } else {
                " (irregular)"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require constructing MatData instances
    // Integration tests will cover this with real MAT files
}
