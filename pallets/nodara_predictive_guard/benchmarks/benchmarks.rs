#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_prediction {
        // No additional setup required.
    }: {
        <pallet::Pallet<T>>::initialize_prediction()?;
    }
    verify {
        let state = <pallet::PredictiveStateStorage<T>>::get();
        assert_eq!(state.parameter, T::BaselineParameter::get());
    }

    analyze_and_predict {
        <pallet::Pallet<T>>::initialize_prediction()?;
        let signal: u32 = 50;
    }: {
        <pallet::Pallet<T>>::analyze_and_predict(signal)?;
    }
    verify {
        let state = <pallet::PredictiveStateStorage<T>>::get();
        let expected = T::BaselineParameter::get().saturating_add(signal / T::PredictionSmoothingFactor::get());
        assert_eq!(state.parameter, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
