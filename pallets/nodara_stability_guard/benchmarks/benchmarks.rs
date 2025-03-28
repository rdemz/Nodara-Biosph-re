#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_stability {
        // Setup before calling initialize_stability
    }: {
        <pallet::Pallet<T>>::initialize_stability()?;
    }
    verify {
        let state = <pallet::StabilityStateStorage<T>>::get();
        assert_eq!(state.parameter, T::BaselineStability::get());
    }

    update_stability {
        <pallet::Pallet<T>>::initialize_stability()?;
        let signal: u32 = 50;
    }: {
        <pallet::Pallet<T>>::update_stability(signal)?;
    }
    verify {
        let state = <pallet::StabilityStateStorage<T>>::get();
        let expected = T::BaselineStability::get().saturating_add(signal.div(10));
        assert_eq!(state.parameter, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
