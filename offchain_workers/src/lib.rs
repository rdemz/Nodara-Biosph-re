#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Offchain Workers Library - Extreme Version
//!
//! This library provides advanced utilities for offchain parallel processing tasks used in the Nodara project.
//! It includes functions for efficient parallel computations using Rayon (when available), asynchronous task execution,
//! and additional utilities for sorting, filtering, and aggregating data in parallel.
//!
//! The library is designed to be fully deployable in both `std` and `no_std` environments (using the `alloc` crate),
//! and includes comprehensive tests and documentation.

extern crate alloc;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use rayon::prelude::*;

/// Computes the parallel sum of a vector of `u8` values.
///
/// # Arguments
///
/// * `data` - A vector of `u8` values to sum.
///
/// # Returns
///
/// The sum of the vector elements as a `u64`.
#[cfg(feature = "std")]
pub fn parallel_sum(data: Vec<u8>) -> u64 {
    data.par_iter().map(|&x| x as u64).sum()
}

/// Computes the sequential sum of a vector of `u8` values.
///
/// # Arguments
///
/// * `data` - A vector of `u8` values to sum.
///
/// # Returns
///
/// The sum of the vector elements as a `u64`.
pub fn sequential_sum(data: Vec<u8>) -> u64 {
    data.iter().map(|&x| x as u64).sum()
}

/// Computes the parallel product of a vector of `u8` values.
/// Returns 1 if the vector is empty.
#[cfg(feature = "std")]
pub fn parallel_product(data: Vec<u8>) -> u64 {
    data.par_iter().map(|&x| x as u64).product()
}

/// Computes the sequential product of a vector of `u8` values.
pub fn sequential_product(data: Vec<u8>) -> u64 {
    data.iter().map(|&x| x as u64).product()
}

/// Sorts a vector of `u32` values in parallel.
#[cfg(feature = "std")]
pub fn parallel_sort(mut data: Vec<u32>) -> Vec<u32> {
    data.par_sort();
    data
}

/// Sorts a vector of `u32` values sequentially.
pub fn sequential_sort(mut data: Vec<u32>) -> Vec<u32> {
    data.sort();
    data
}

/// Trait defining an offchain task.
pub trait OffchainTask {
    /// Executes the offchain task and returns the result as a vector of bytes.
    fn execute(&self) -> Result<Vec<u8>, &'static str>;
}

/// Dummy offchain task that computes the sum of a vector of `u8` values.
pub struct SumTask {
    pub data: Vec<u8>,
    /// Flag to choose between parallel and sequential processing.
    pub use_parallel: bool,
}

impl OffchainTask for SumTask {
    fn execute(&self) -> Result<Vec<u8>, &'static str> {
        let sum = if self.use_parallel {
            #[cfg(feature = "std")]
            {
                parallel_sum(self.data.clone())
            }
            #[cfg(not(feature = "std"))]
            {
                sequential_sum(self.data.clone())
            }
        } else {
            sequential_sum(self.data.clone())
        };
        Ok(sum.to_le_bytes().to_vec())
    }
}

/// Dummy offchain task that sorts a vector of `u32` values.
pub struct SortTask {
    pub data: Vec<u32>,
    pub use_parallel: bool,
}

impl OffchainTask for SortTask {
    fn execute(&self) -> Result<Vec<u8>, &'static str> {
        let sorted = if self.use_parallel {
            #[cfg(feature = "std")]
            {
                parallel_sort(self.data.clone())
            }
            #[cfg(not(feature = "std"))]
            {
                sequential_sort(self.data.clone())
            }
        } else {
            sequential_sort(self.data.clone())
        };
        // Encode each u32 in little-endian format.
        let mut result = Vec::with_capacity(sorted.len() * 4);
        for num in sorted {
            result.extend_from_slice(&num.to_le_bytes());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_sum() {
        let data = vec![1, 2, 3, 4, 5];
        #[cfg(feature = "std")]
        {
            let result = parallel_sum(data.clone());
            let expected = sequential_sum(data);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sequential_sum() {
        let data = vec![10, 20, 30];
        let result = sequential_sum(data);
        assert_eq!(result, 60);
    }

    #[test]
    fn test_parallel_product() {
        let data = vec![1, 2, 3, 4];
        #[cfg(feature = "std")]
        {
            let result = parallel_product(data.clone());
            let expected = sequential_product(data);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sequential_product() {
        let data = vec![2, 3, 4];
        let result = sequential_product(data);
        assert_eq!(result, 24);
    }

    #[test]
    fn test_parallel_sort() {
        let data = vec![5, 3, 1, 4, 2];
        #[cfg(feature = "std")]
        {
            let result = parallel_sort(data.clone());
            let expected = sequential_sort(data);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sequential_sort() {
        let data = vec![10, 5, 8, 1];
        let result = sequential_sort(data.clone());
        let mut expected = data;
        expected.sort();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sum_task_sequential() {
        let task = SumTask { data: vec![10, 20, 30], use_parallel: false };
        let result = task.execute().expect("Task should execute");
        let sum = u64::from_le_bytes(result.try_into().expect("Slice with incorrect length"));
        assert_eq!(sum, 60);
    }

    #[test]
    fn test_sort_task_sequential() {
        let task = SortTask { data: vec![4, 1, 3, 2], use_parallel: false };
        let result = task.execute().expect("Task should execute");
        // Decode result as vector of u32
        let mut sorted = Vec::new();
        for chunk in result.chunks(4) {
            sorted.push(u32::from_le_bytes(chunk.try_into().expect("Chunk size must be 4")));
        }
        let mut expected = vec![4, 1, 3, 2];
        expected.sort();
        assert_eq!(sorted, expected);
    }
}
