//! Store for DDSketch
//!
//! This module provides the storage backend for DDSketch, handling the
//! mapping from indices to counts.

use std::collections::HashMap;
use crate::Result;

/// Trait for storing index-count pairs
pub trait Store {
    /// Add a count to the given index
    fn add(&mut self, index: i32, count: u64);
    
    /// Get the count for a given index
    fn get(&self, index: i32) -> u64;
    
    /// Get the total count across all indices
    fn total_count(&self) -> u64;
    
    /// Check if the store is empty
    fn is_empty(&self) -> bool;
    
    /// Get the minimum index with a non-zero count
    fn min_index(&self) -> Option<i32>;
    
    /// Get the maximum index with a non-zero count
    fn max_index(&self) -> Option<i32>;
    
    /// Iterate over all (index, count) pairs
    fn iter(&self) -> Box<dyn Iterator<Item = (i32, u64)> + '_>;
    
    /// Merge another store into this one
    fn merge(&mut self, other: &dyn Store);
    
    /// Clear all data
    fn clear(&mut self);
}

/// A simple HashMap-based store
#[derive(Debug, Clone)]
pub struct DenseStore {
    bins: HashMap<i32, u64>,
    total_count: u64,
}

impl DenseStore {
    /// Create a new empty store
    pub fn new() -> Self {
        DenseStore {
            bins: HashMap::new(),
            total_count: 0,
        }
    }
    
    /// Create a store with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        DenseStore {
            bins: HashMap::with_capacity(capacity),
            total_count: 0,
        }
    }
}

impl Default for DenseStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for DenseStore {
    fn add(&mut self, index: i32, count: u64) {
        if count == 0 {
            return;
        }
        
        *self.bins.entry(index).or_insert(0) += count;
        self.total_count += count;
    }
    
    fn get(&self, index: i32) -> u64 {
        self.bins.get(&index).copied().unwrap_or(0)
    }
    
    fn total_count(&self) -> u64 {
        self.total_count
    }
    
    fn is_empty(&self) -> bool {
        self.total_count == 0
    }
    
    fn min_index(&self) -> Option<i32> {
        self.bins.keys().min().copied()
    }
    
    fn max_index(&self) -> Option<i32> {
        self.bins.keys().max().copied()
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (i32, u64)> + '_> {
        Box::new(self.bins.iter().map(|(&index, &count)| (index, count)))
    }
    
    fn merge(&mut self, other: &dyn Store) {
        for (index, count) in other.iter() {
            self.add(index, count);
        }
    }
    
    fn clear(&mut self) {
        self.bins.clear();
        self.total_count = 0;
    }
}

/// A collapsing store that maintains a maximum number of bins
#[derive(Debug, Clone)]
pub struct CollapsingStore {
    store: DenseStore,
    max_num_bins: usize,
}

impl CollapsingStore {
    /// Create a new collapsing store with the given maximum number of bins
    pub fn new(max_num_bins: usize) -> Self {
        CollapsingStore {
            store: DenseStore::with_capacity(max_num_bins),
            max_num_bins,
        }
    }
    
    /// Collapse bins if necessary to maintain the maximum number of bins
    fn collapse_if_needed(&mut self) {
        if self.store.bins.len() <= self.max_num_bins {
            return;
        }
        
        // Simple collapsing strategy: merge adjacent bins
        let mut sorted_indices: Vec<i32> = self.store.bins.keys().copied().collect();
        sorted_indices.sort_unstable();
        
        while sorted_indices.len() > self.max_num_bins {
            // Find the pair of adjacent bins with the smallest combined count
            let mut min_combined_count = u64::MAX;
            let mut merge_index = 0;
            
            for i in 0..sorted_indices.len() - 1 {
                let count1 = self.store.get(sorted_indices[i]);
                let count2 = self.store.get(sorted_indices[i + 1]);
                let combined = count1 + count2;
                
                if combined < min_combined_count {
                    min_combined_count = combined;
                    merge_index = i;
                }
            }
            
            // Merge the bins
            let index1 = sorted_indices[merge_index];
            let index2 = sorted_indices[merge_index + 1];
            let count1 = self.store.get(index1);
            let count2 = self.store.get(index2);
            
            // Remove both bins
            self.store.bins.remove(&index1);
            self.store.bins.remove(&index2);
            
            // Add combined count to the lower index
            self.store.bins.insert(index1, count1 + count2);
            
            // Update sorted indices
            sorted_indices.remove(merge_index + 1);
        }
    }
}

impl Store for CollapsingStore {
    fn add(&mut self, index: i32, count: u64) {
        self.store.add(index, count);
        self.collapse_if_needed();
    }
    
    fn get(&self, index: i32) -> u64 {
        self.store.get(index)
    }
    
    fn total_count(&self) -> u64 {
        self.store.total_count()
    }
    
    fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
    
    fn min_index(&self) -> Option<i32> {
        self.store.min_index()
    }
    
    fn max_index(&self) -> Option<i32> {
        self.store.max_index()
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (i32, u64)> + '_> {
        self.store.iter()
    }
    
    fn merge(&mut self, other: &dyn Store) {
        self.store.merge(other);
        self.collapse_if_needed();
    }
    
    fn clear(&mut self) {
        self.store.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dense_store_basic_operations() {
        let mut store = DenseStore::new();
        
        assert!(store.is_empty());
        assert_eq!(store.total_count(), 0);
        assert_eq!(store.min_index(), None);
        assert_eq!(store.max_index(), None);
        
        store.add(10, 5);
        store.add(20, 3);
        store.add(10, 2); // Should add to existing
        
        assert!(!store.is_empty());
        assert_eq!(store.total_count(), 10);
        assert_eq!(store.get(10), 7);
        assert_eq!(store.get(20), 3);
        assert_eq!(store.get(30), 0);
        assert_eq!(store.min_index(), Some(10));
        assert_eq!(store.max_index(), Some(20));
    }
    
    #[test]
    fn test_dense_store_merge() {
        let mut store1 = DenseStore::new();
        let mut store2 = DenseStore::new();
        
        store1.add(10, 5);
        store1.add(20, 3);
        
        store2.add(10, 2);
        store2.add(30, 4);
        
        store1.merge(&store2);
        
        assert_eq!(store1.total_count(), 14);
        assert_eq!(store1.get(10), 7);
        assert_eq!(store1.get(20), 3);
        assert_eq!(store1.get(30), 4);
    }
    
    #[test]
    fn test_collapsing_store() {
        let mut store = CollapsingStore::new(2);
        
        store.add(10, 5);
        store.add(20, 3);
        assert_eq!(store.store.bins.len(), 2);
        
        // This should trigger collapsing
        store.add(30, 2);
        assert!(store.store.bins.len() <= 2);
        assert_eq!(store.total_count(), 10);
    }
}
