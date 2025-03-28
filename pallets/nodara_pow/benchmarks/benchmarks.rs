#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_pow {
        // Setup: no special setup required
    }: {
        <pallet::Pallet<T>>::initialize_pow()?;
    }
    verify {
        let state = <pallet::PowStateStorage<T>>::get();
        assert_eq!(state.difficulty, T::BaselineDifficulty::get());
    }

    submit_work {
        <pallet::Pallet<T>>::initialize_pow()?;
        let work_value: u32 = T::BaselineDifficulty::get(); // Use baseline value for valid work
        let signature: Vec<u8> = b"BenchmarkSignature".to_vec();
    }: {
        <pallet::Pallet<T>>::submit_work(RawOrigin::Signed(account("miner", 0, 0)).into(), work_value, signature.clone())?;
    }
    verify {
        // Verify that work submission updated total work accordingly
        let state = <pallet::PowStateStorage<T>>::get();
        assert!(state.total_work >= work_value);
    }

    adjust_difficulty {
        <pallet::Pallet<T>>::initialize_pow()?;
        let signal: u32 = 50;
    }: {
        <pallet::Pallet<T>>::adjust_difficulty(signal)?;
    }
    verify {
        let state = <pallet::PowStateStorage<T>>::get();
        let expected = T::BaselineDifficulty::get().saturating_add(signal / T::PowSmoothingFactor::get());
        assert_eq!(state.difficulty, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
