//! MAT file loading and variable listing.
//!
//! This module provides [`MatFile`], a wrapper around the matfile crate
//! for loading and inspecting MATLAB/Octave .mat files.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use matfile::MatFile as RawMatFile;

use crate::error::{Error, Result};
use super::data::MatData;

/// A loaded MAT file containing numeric variables.
///
/// `MatFile` wraps the matfile crate's parser and provides convenient
/// access to numeric arrays within the file.
///
/// # Supported Formats
///
/// - Level 5 MAT files (MATLAB v5, v6, v7)
/// - v7 compressed files (zlib)
/// - Numeric arrays: double, single, int8/16/32/64, uint8/16/32/64
/// - Complex arrays (stored as two separate real arrays)
///
/// # Unsupported
///
/// - Level 4 MAT files (legacy format)
/// - HDF5-based v7.3 files
/// - Cell arrays, structs, sparse matrices, function handles
///
/// # Example
///
/// ```no_run
/// use sdif_rs::MatFile;
///
/// let mat = MatFile::open("analysis.mat")?;
///
/// // List all variables
/// for name in mat.variable_names() {
///     println!("Variable: {}", name);
/// }
///
/// // Get a specific variable
/// if let Some(data) = mat.get("frequencies") {
///     println!("Shape: {:?}", data.shape());
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct MatFile {
    /// Parsed variables, keyed by name.
    variables: HashMap<String, MatData>,

    /// Original file path (for error messages).
    path: String,
}

impl MatFile {
    /// Open and parse a MAT file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .mat file.
    ///
    /// # Returns
    ///
    /// A `MatFile` containing all parseable numeric variables.
    ///
    /// # Errors
    ///
    /// - [`Error::Io`] if the file cannot be read
    /// - [`Error::InvalidFormat`] if the file is not a valid MAT file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::MatFile;
    ///
    /// let mat = MatFile::open("data.mat")?;
    /// println!("Loaded {} variables", mat.len());
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        let file = File::open(path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open MAT file '{}': {}", path_str, e),
            ))
        })?;

        let reader = BufReader::new(file);

        let mat_file = RawMatFile::parse(reader).map_err(|e| {
            Error::invalid_format(format!("Failed to parse MAT file '{}': {}", path_str, e))
        })?;

        let mut variables = HashMap::new();

        for array in mat_file.arrays() {
            let name = array.name().to_string();

            // Try to convert to MatData
            match MatData::from_matfile_array(array) {
                Ok(data) => {
                    variables.insert(name, data);
                }
                Err(e) => {
                    // Log but don't fail - skip unsupported variable types
                    eprintln!("Warning: Skipping variable '{}': {}", name, e);
                }
            }
        }

        Ok(MatFile {
            variables,
            path: path_str,
        })
    }

    /// Get the names of all numeric variables in the file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// for name in mat.variable_names() {
    ///     println!("Found variable: {}", name);
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn variable_names(&self) -> impl Iterator<Item = &str> {
        self.variables.keys().map(|s| s.as_str())
    }

    /// Get a variable by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// The variable data if found, or `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// if let Some(freqs) = mat.get("frequencies") {
    ///     println!("Frequencies shape: {:?}", freqs.shape());
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn get(&self, name: &str) -> Option<&MatData> {
        self.variables.get(name)
    }

    /// Get a variable by name, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFormat`] if the variable doesn't exist.
    pub fn require(&self, name: &str) -> Result<&MatData> {
        self.get(name).ok_or_else(|| {
            Error::invalid_format(format!(
                "Variable '{}' not found in MAT file '{}'",
                name, self.path
            ))
        })
    }

    /// Get the number of numeric variables in the file.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if the file contains no numeric variables.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Get the file path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Iterate over all variables.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MatData)> {
        self.variables.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Find variables that look like time vectors.
    ///
    /// Uses heuristics to identify potential time vectors:
    /// - Named "time", "t", "times", etc.
    /// - 1D arrays with ascending values
    /// - Values starting near zero
    ///
    /// # Returns
    ///
    /// Names of variables that appear to be time vectors.
    pub fn find_time_vectors(&self) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, data)| data.is_likely_time_vector())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get detailed information about all variables (for --list mode).
    ///
    /// Returns a formatted string describing each variable.
    pub fn describe(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Variables in '{}':", self.path));
        lines.push(String::new());

        let mut names: Vec<_> = self.variable_names().collect();
        names.sort();

        // Calculate column widths
        let max_name_len = names.iter().map(|n| n.len()).max().unwrap_or(4);

        lines.push(format!(
            "  {:<width$}  {:>12}  {:>8}  {}",
            "Name",
            "Shape",
            "Type",
            "Notes",
            width = max_name_len
        ));
        lines.push(format!(
            "  {:-<width$}  {:->12}  {:->8}  -----",
            "", "", "",
            width = max_name_len
        ));

        for name in names {
            if let Some(data) = self.get(name) {
                let shape_str = format!("{:?}", data.shape());
                let type_str = if data.is_complex() { "complex" } else { "real" };

                let mut notes = Vec::new();
                if data.is_likely_time_vector() {
                    notes.push("time?");
                }
                if data.is_1d() {
                    notes.push("1D");
                }

                lines.push(format!(
                    "  {:<width$}  {:>12}  {:>8}  {}",
                    name,
                    shape_str,
                    type_str,
                    notes.join(", "),
                    width = max_name_len
                ));
            }
        }

        lines.join("\n")
    }
}

impl IntoIterator for MatFile {
    type Item = (String, MatData);
    type IntoIter = std::collections::hash_map::IntoIter<String, MatData>;

    fn into_iter(self) -> Self::IntoIter {
        self.variables.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_nonexistent() {
        let result = MatFile::open("/nonexistent/file.mat");
        assert!(result.is_err());
    }

    // Additional tests require test MAT files
}
