#![cfg_attr(not(feature = "std"), no_std)]

//! # Offchain Workers Library for Nodara BIOSPHÈRE QUANTIC
//!
//! This library provides utilities for offchain parallel processing tasks used in the Nodara project.
//! It includes example functions that leverage Rayon for efficient parallel computations.
//!
//! ## Features
//! - Parallel processing utilities (e.g., parallel sum calculation)
//! - Sequential implementations for comparison
//!
//! Additional offchain processing functions can be added as needed.

extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "std")]
extern crate rayon;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_sum() {
        let data = vec![1, 2, 3, 4, 5];
        let result = parallel_sum(data.clone());
        let expected = sequential_sum(data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sequential_sum() {
        let data = vec![10, 20, 30];
        let result = sequential_sum(data);
        assert_eq!(result, 60);
    }
}
