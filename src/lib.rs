//! # DDSketch: Distributed Quantile Sketch
//!
//! This crate provides a Rust implementation of DDSketch, a distributed quantile sketch
//! that provides relative-error guarantees for quantile estimation.
//!
//! DDSketch is particularly useful for:
//! - Estimating quantiles (percentiles) of streaming data
//! - Memory-efficient approximate quantile computation
//! - Distributed systems where sketches need to be merged
//!
//! ## Example
//!
//! ```
//! use ddsketch_rs::DDSketch;
//!
//! let mut sketch = DDSketch::new(0.02).unwrap(); // 2% relative accuracy
//! 
//! // Add values
//! sketch.add(1.0);
//! sketch.add(2.0);
//! sketch.add(3.0);
//! 
//! // Get quantiles
//! let median = sketch.get_quantile_value(0.5).unwrap();
//! let p99 = sketch.get_quantile_value(0.99).unwrap();
//! ```

pub mod ddsketch;
pub mod store;
pub mod mapping;

pub use ddsketch::DDSketch;
pub use store::Store;
pub use mapping::IndexMapping;

/// Errors that can occur in DDSketch operations
#[derive(Debug, Clone, PartialEq)]
pub enum DDSketchError {
    /// Invalid relative accuracy (must be between 0 and 1)
    InvalidRelativeAccuracy,
    /// Invalid quantile value (must be between 0 and 1)
    InvalidQuantile,
    /// Empty sketch (no values added)
    EmptySketch,
    /// Incompatible sketches for merging
    IncompatibleSketches,
}

impl std::fmt::Display for DDSketchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DDSketchError::InvalidRelativeAccuracy => {
                write!(f, "Relative accuracy must be between 0 and 1")
            }
            DDSketchError::InvalidQuantile => {
                write!(f, "Quantile must be between 0 and 1")
            }
            DDSketchError::EmptySketch => {
                write!(f, "Cannot compute quantile from empty sketch")
            }
            DDSketchError::IncompatibleSketches => {
                write!(f, "Sketches are incompatible for merging")
            }
        }
    }
}

impl std::error::Error for DDSketchError {}

/// Result type for DDSketch operations
pub type Result<T> = std::result::Result<T, DDSketchError>;
