#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_growth {
        // Setup before initializing growth
    }: {
        <pallet::Pallet<T>>::initialize_growth()?;
    }
    verify {
        let state = <pallet::GrowthStateStorage<T>>::get();
        assert_eq!(state.multiplier, T::BaselineMultiplier::get());
    }

    update_multiplier {
        <pallet::Pallet<T>>::initialize_growth()?;
        let signal: u32 = 50;
    }: {
        <pallet::Pallet<T>>::update_multiplier(signal)?;
    }
    verify {
        let state = <pallet::GrowthStateStorage<T>>::get();
        let expected = T::BaselineMultiplier::get().saturating_add(signal.div(10));
        assert_eq!(state.multiplier, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
