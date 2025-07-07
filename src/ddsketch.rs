//! Core DDSketch implementation
//!
//! This module provides the main DDSketch data structure with methods for
//! adding values, computing quantiles, and merging sketches.

use crate::{DDSketchError, Result};
use crate::mapping::{IndexMapping, LogarithmicMapping};
use crate::store::{Store, DenseStore, CollapsingStore};
use std::fmt;

/// The main DDSketch data structure
///
/// DDSketch provides approximate quantile estimation with relative error guarantees.
/// It uses a logarithmic mapping to achieve this while maintaining mergeable sketches.
pub struct DDSketch {
    /// The mapping from values to indices
    mapping: Box<dyn IndexMapping>,
    
    /// The store containing counts for each index
    store: Box<dyn Store>,
    
    /// Count of zero values
    zero_count: u64,
    
    /// Count of negative values (stored as their absolute value)
    negative_store: Box<dyn Store>,
    
    /// The minimum value seen
    min_value: Option<f64>,
    
    /// The maximum value seen
    max_value: Option<f64>,
}

impl DDSketch {
    /// Create a new DDSketch with the given relative accuracy
    ///
    /// # Arguments
    /// * `relative_accuracy` - The relative accuracy parameter (between 0 and 1)
    ///
    /// # Returns
    /// A new DDSketch instance
    pub fn new(relative_accuracy: f64) -> Result<Self> {
        let mapping = LogarithmicMapping::new(relative_accuracy)?;
        
        Ok(DDSketch {
            mapping: Box::new(mapping),
            store: Box::new(DenseStore::new()),
            zero_count: 0,
            negative_store: Box::new(DenseStore::new()),
            min_value: None,
            max_value: None,
        })
    }
    
    /// Create a new DDSketch with the given relative accuracy and maximum number of bins
    ///
    /// # Arguments
    /// * `relative_accuracy` - The relative accuracy parameter (between 0 and 1)
    /// * `max_num_bins` - The maximum number of bins to maintain
    ///
    /// # Returns
    /// A new DDSketch instance with collapsing stores
    pub fn with_max_bins(relative_accuracy: f64, max_num_bins: usize) -> Result<Self> {
        let mapping = LogarithmicMapping::new(relative_accuracy)?;
        
        Ok(DDSketch {
            mapping: Box::new(mapping),
            store: Box::new(CollapsingStore::new(max_num_bins)),
            zero_count: 0,
            negative_store: Box::new(CollapsingStore::new(max_num_bins)),
            min_value: None,
            max_value: None,
        })
    }
    
    /// Add a value to the sketch
    ///
    /// # Arguments
    /// * `value` - The value to add
    pub fn add(&mut self, value: f64) {
        self.add_with_count(value, 1);
    }
    
    /// Add a value with a specific count to the sketch
    ///
    /// # Arguments
    /// * `value` - The value to add
    /// * `count` - The number of times to add the value
    pub fn add_with_count(&mut self, value: f64, count: u64) {
        if count == 0 {
            return;
        }
        
        // Update min/max
        self.min_value = Some(self.min_value.map_or(value, |min| min.min(value)));
        self.max_value = Some(self.max_value.map_or(value, |max| max.max(value)));
        
        if value == 0.0 {
            self.zero_count += count;
        } else if value > 0.0 {
            if let Ok(index) = self.mapping.key(value) {
                self.store.add(index, count);
            }
        } else {
            // Handle negative values by storing their absolute value
            if let Ok(index) = self.mapping.key(-value) {
                self.negative_store.add(index, count);
            }
        }
    }
    
    /// Get the total count of values in the sketch
    pub fn count(&self) -> u64 {
        self.store.total_count() + self.zero_count + self.negative_store.total_count()
    }
    
    /// Check if the sketch is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
    
    /// Get the minimum value in the sketch
    pub fn min(&self) -> Option<f64> {
        self.min_value
    }
    
    /// Get the maximum value in the sketch
    pub fn max(&self) -> Option<f64> {
        self.max_value
    }
    
    /// Get the relative accuracy of the sketch
    pub fn relative_accuracy(&self) -> f64 {
        self.mapping.relative_accuracy()
    }
    
    /// Get the value at a given quantile
    ///
    /// # Arguments
    /// * `quantile` - The quantile to query (between 0 and 1)
    ///
    /// # Returns
    /// The estimated value at the given quantile
    pub fn get_quantile_value(&self, quantile: f64) -> Result<f64> {
        if quantile < 0.0 || quantile > 1.0 {
            return Err(DDSketchError::InvalidQuantile);
        }
        
        if self.is_empty() {
            return Err(DDSketchError::EmptySketch);
        }
        
        let total_count = self.count();
        let rank = (quantile * total_count as f64) as u64;
        
        // Find the value at the given rank
        let mut current_rank = 0u64;
        
        // Check negative values first (in reverse order)
        if !self.negative_store.is_empty() {
            let mut negative_indices: Vec<i32> = self.negative_store.iter().map(|(i, _)| i).collect();
            negative_indices.sort_by(|a, b| b.cmp(a)); // Reverse order for negative values
            
            for index in negative_indices {
                let count = self.negative_store.get(index);
                if current_rank + count > rank {
                    return Ok(-self.mapping.value(index));
                }
                current_rank += count;
            }
        }
        
        // Check zero values
        if current_rank + self.zero_count > rank {
            return Ok(0.0);
        }
        current_rank += self.zero_count;
        
        // Check positive values
        if !self.store.is_empty() {
            let mut positive_indices: Vec<i32> = self.store.iter().map(|(i, _)| i).collect();
            positive_indices.sort();
            
            for index in positive_indices {
                let count = self.store.get(index);
                if current_rank + count > rank {
                    return Ok(self.mapping.value(index));
                }
                current_rank += count;
            }
        }
        
        // Should not reach here if counts are correct
        self.max_value.ok_or(DDSketchError::EmptySketch)
    }
    
    /// Get values for multiple quantiles
    ///
    /// # Arguments
    /// * `quantiles` - A slice of quantiles to query
    ///
    /// # Returns
    /// A vector of estimated values for the given quantiles
    pub fn get_quantile_values(&self, quantiles: &[f64]) -> Result<Vec<f64>> {
        quantiles.iter()
            .map(|&q| self.get_quantile_value(q))
            .collect()
    }
    
    /// Merge another sketch into this one
    ///
    /// # Arguments
    /// * `other` - The other sketch to merge
    ///
    /// # Returns
    /// An error if the sketches are incompatible
    pub fn merge(&mut self, other: &DDSketch) -> Result<()> {
        // Check compatibility
        if (self.mapping.relative_accuracy() - other.mapping.relative_accuracy()).abs() > 1e-10 {
            return Err(DDSketchError::IncompatibleSketches);
        }
        
        // Merge stores
        self.store.merge(other.store.as_ref());
        self.negative_store.merge(other.negative_store.as_ref());
        self.zero_count += other.zero_count;
        
        // Update min/max
        if let Some(other_min) = other.min_value {
            self.min_value = Some(self.min_value.map_or(other_min, |min| min.min(other_min)));
        }
        
        if let Some(other_max) = other.max_value {
            self.max_value = Some(self.max_value.map_or(other_max, |max| max.max(other_max)));
        }
        
        Ok(())
    }
    
    /// Clear all data from the sketch
    pub fn clear(&mut self) {
        self.store.clear();
        self.negative_store.clear();
        self.zero_count = 0;
        self.min_value = None;
        self.max_value = None;
    }
}

impl fmt::Debug for DDSketch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DDSketch")
            .field("relative_accuracy", &self.mapping.relative_accuracy())
            .field("count", &self.count())
            .field("min_value", &self.min_value)
            .field("max_value", &self.max_value)
            .finish()
    }
}

impl Clone for DDSketch {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone that creates a new sketch with the same parameters
        // In a real implementation, you might want to implement Clone for the trait objects
        let mut cloned = DDSketch::new(self.mapping.relative_accuracy()).unwrap();
        
        // Copy the data by iterating through the stores
        for (index, count) in self.store.iter() {
            cloned.store.add(index, count);
        }
        
        for (index, count) in self.negative_store.iter() {
            cloned.negative_store.add(index, count);
        }
        
        cloned.zero_count = self.zero_count;
        cloned.min_value = self.min_value;
        cloned.max_value = self.max_value;
        
        cloned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ddsketch_creation() {
        let sketch = DDSketch::new(0.02).unwrap();
        assert_eq!(sketch.relative_accuracy(), 0.02);
        assert!(sketch.is_empty());
        assert_eq!(sketch.count(), 0);
        
        // Invalid relative accuracy
        assert!(DDSketch::new(0.0).is_err());
        assert!(DDSketch::new(1.0).is_err());
    }
    
    #[test]
    fn test_ddsketch_add_values() {
        let mut sketch = DDSketch::new(0.02).unwrap();
        
        sketch.add(1.0);
        sketch.add(2.0);
        sketch.add(3.0);
        
        assert_eq!(sketch.count(), 3);
        assert_eq!(sketch.min(), Some(1.0));
        assert_eq!(sketch.max(), Some(3.0));
        
        // Test adding zero and negative values
        sketch.add(0.0);
        sketch.add(-1.0);
        
        assert_eq!(sketch.count(), 5);
        assert_eq!(sketch.min(), Some(-1.0));
        assert_eq!(sketch.max(), Some(3.0));
    }
    
    #[test]
    fn test_ddsketch_quantiles() {
        let mut sketch = DDSketch::new(0.02).unwrap();
        
        // Add values 1 through 100
        for i in 1..=100 {
            sketch.add(i as f64);
        }
        
        let median = sketch.get_quantile_value(0.5).unwrap();
        let p90 = sketch.get_quantile_value(0.9).unwrap();
        let p99 = sketch.get_quantile_value(0.99).unwrap();
        
        // Check that quantiles are reasonable
        assert!(median > 40.0 && median < 60.0);
        assert!(p90 > 80.0 && p90 < 95.0);
        assert!(p99 > 95.0 && p99 <= 100.0);
        
        // Test multiple quantiles
        let quantiles = vec![0.25, 0.5, 0.75, 0.95];
        let values = sketch.get_quantile_values(&quantiles).unwrap();
        assert_eq!(values.len(), 4);
        
        // Values should be increasing
        for i in 1..values.len() {
            assert!(values[i] >= values[i-1]);
        }
    }
    
    #[test]
    fn test_ddsketch_merge() {
        let mut sketch1 = DDSketch::new(0.02).unwrap();
        let mut sketch2 = DDSketch::new(0.02).unwrap();
        
        // Add different values to each sketch
        for i in 1..=50 {
            sketch1.add(i as f64);
        }
        
        for i in 51..=100 {
            sketch2.add(i as f64);
        }
        
        let count1 = sketch1.count();
        let count2 = sketch2.count();
        
        sketch1.merge(&sketch2).unwrap();
        
        assert_eq!(sketch1.count(), count1 + count2);
        assert_eq!(sketch1.min(), Some(1.0));
        assert_eq!(sketch1.max(), Some(100.0));
        
        // Test incompatible sketches
        let mut sketch3 = DDSketch::new(0.01).unwrap();
        sketch3.add(1.0);
        
        assert!(sketch1.merge(&sketch3).is_err());
    }
    
    #[test]
    fn test_ddsketch_edge_cases() {
        let sketch = DDSketch::new(0.02).unwrap();
        
        // Empty sketch
        assert!(sketch.get_quantile_value(0.5).is_err());
        
        // Invalid quantiles
        let mut sketch = DDSketch::new(0.02).unwrap();
        sketch.add(1.0);
        
        assert!(sketch.get_quantile_value(-0.1).is_err());
        assert!(sketch.get_quantile_value(1.1).is_err());
    }
}
