//! Index mapping for DDSketch
//!
//! This module provides the logarithmic index mapping that is core to DDSketch's
//! relative error guarantees.

use crate::{DDSketchError, Result};

/// Trait for mapping values to indices
pub trait IndexMapping {
    /// Map a value to its corresponding index
    fn key(&self, value: f64) -> Result<i32>;
    
    /// Get the value corresponding to an index
    fn value(&self, index: i32) -> f64;
    
    /// Get the relative accuracy of this mapping
    fn relative_accuracy(&self) -> f64;
    
    /// Get the minimum possible index
    fn min_possible_index(&self) -> i32;
    
    /// Get the maximum possible index
    fn max_possible_index(&self) -> i32;
}

/// Logarithmic index mapping
///
/// This mapping uses a logarithmic scale to map values to indices, which provides
/// the relative error guarantees that DDSketch is known for.
#[derive(Debug, Clone)]
pub struct LogarithmicMapping {
    /// The relative accuracy parameter
    relative_accuracy: f64,
    /// The multiplier for the logarithmic mapping
    multiplier: f64,
    /// The offset for the logarithmic mapping
    offset: f64,
}

impl LogarithmicMapping {
    /// Create a new logarithmic mapping with the given relative accuracy
    pub fn new(relative_accuracy: f64) -> Result<Self> {
        if relative_accuracy <= 0.0 || relative_accuracy >= 1.0 {
            return Err(DDSketchError::InvalidRelativeAccuracy);
        }
        
        let multiplier = 1.0 / (1.0 + relative_accuracy).ln();
        let offset = 0.0;
        
        Ok(LogarithmicMapping {
            relative_accuracy,
            multiplier,
            offset,
        })
    }
}

impl IndexMapping for LogarithmicMapping {
    fn key(&self, value: f64) -> Result<i32> {
        if value <= 0.0 {
            return Ok(i32::MIN);
        }
        
        let index = (value.ln() * self.multiplier + self.offset).floor() as i32;
        Ok(index)
    }
    
    fn value(&self, index: i32) -> f64 {
        if index == i32::MIN {
            return 0.0;
        }
        
        ((index as f64 - self.offset) / self.multiplier).exp()
    }
    
    fn relative_accuracy(&self) -> f64 {
        self.relative_accuracy
    }
    
    fn min_possible_index(&self) -> i32 {
        i32::MIN
    }
    
    fn max_possible_index(&self) -> i32 {
        i32::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logarithmic_mapping_creation() {
        let mapping = LogarithmicMapping::new(0.01).unwrap();
        assert_eq!(mapping.relative_accuracy(), 0.01);
        
        // Invalid relative accuracy
        assert!(LogarithmicMapping::new(0.0).is_err());
        assert!(LogarithmicMapping::new(1.0).is_err());
        assert!(LogarithmicMapping::new(-0.1).is_err());
    }
    
    #[test]
    fn test_mapping_properties() {
        let mapping = LogarithmicMapping::new(0.02).unwrap();
        
        // Test that key and value are inverses for positive values
        let value = 100.0;
        let index = mapping.key(value).unwrap();
        let recovered = mapping.value(index);
        
        // The relative error should be within the specified accuracy
        let relative_error = (recovered - value).abs() / value;
        assert!(relative_error <= mapping.relative_accuracy());
        
        // Test zero and negative values
        assert_eq!(mapping.key(0.0).unwrap(), i32::MIN);
        assert_eq!(mapping.key(-1.0).unwrap(), i32::MIN);
        assert_eq!(mapping.value(i32::MIN), 0.0);
    }
    
    #[test]
    fn test_monotonicity() {
        let mapping = LogarithmicMapping::new(0.02).unwrap();
        
        let values = vec![0.1, 1.0, 10.0, 100.0, 1000.0];
        let mut indices = Vec::new();
        
        for &value in &values {
            indices.push(mapping.key(value).unwrap());
        }
        
        for i in 1..indices.len() {
            assert!(indices[i] >= indices[i-1]);
        }
    }
}
