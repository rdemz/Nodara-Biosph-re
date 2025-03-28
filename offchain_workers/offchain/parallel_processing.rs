// offchain/parallel_processing.rs - Legendary Edition
//
// This module demonstrates how to use Rayon for parallel processing of non-deterministic tasks.
// It is intended for offchain computations such as heavy cryptographic verification or data aggregation.

use rayon::prelude::*;
use sp_std::vec::Vec;

/// Computes the sum of a vector of bytes in parallel.
/// This is a simple example to demonstrate parallel processing.
pub fn parallel_sum(data: Vec<u8>) -> u64 {
    data.par_iter().map(|&x| x as u64).sum()
}

/// Example function to compute the average value from a large dataset in parallel.
pub fn parallel_average(data: Vec<u8>) -> f64 {
    let sum: u64 = parallel_sum(data.clone());
    let count = data.len() as f64;
    sum as f64 / count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_sum() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = parallel_sum(data);
        assert_eq!(result, 55);
    }

    #[test]
    fn test_parallel_average() {
        let data = vec![10, 20, 30, 40, 50];
        let avg = parallel_average(data);
        assert!((avg - 30.0).abs() < 0.0001);
    }
}
