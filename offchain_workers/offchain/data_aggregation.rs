// offchain/data_aggregation.rs - Legendary Edition
//
// This module provides functions to aggregate data from multiple sources off-chain.
// It demonstrates how to preprocess and combine data before submitting aggregated results on-chain.

use rayon::prelude::*;
use sp_std::vec::Vec;

/// Aggregates a list of numerical data points and returns their sum and average.
pub fn aggregate_data(data: Vec<Vec<u32>>) -> (u32, f64) {
    // Flatten the 2D vector into a 1D vector using parallel processing
    let flattened: Vec<u32> = data.into_par_iter().flatten().collect();
    let sum: u32 = flattened.par_iter().sum();
    let count = flattened.len() as f64;
    let average = if count > 0.0 { sum as f64 / count } else { 0.0 };
    (sum, average)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_data() {
        let data = vec![
            vec![10, 20, 30],
            vec![40, 50],
            vec![60],
        ];
        let (sum, avg) = aggregate_data(data);
        assert_eq!(sum, 210);
        assert!((avg - 35.0).abs() < 0.0001);
    }
}
