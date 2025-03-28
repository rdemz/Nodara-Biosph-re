#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_reserve {
        // No specific setup required.
    }: {
        <pallet::Pallet<T>>::initialize_reserve()?;
    }
    verify {
        let state = <pallet::ReserveStateStorage<T>>::get();
        assert_eq!(state.balance, T::BaselineReserve::get());
    }

    contribute {
        <pallet::Pallet<T>>::initialize_reserve()?;
        let amount: u32 = 200;
    }: {
        <pallet::Pallet<T>>::contribute(amount, b"Benchmark Contribution".to_vec())?;
    }
    verify {
        let state = <pallet::ReserveStateStorage<T>>::get();
        assert_eq!(state.balance, T::BaselineReserve::get().saturating_add(200));
    }

    update_reserve {
        <pallet::Pallet<T>>::initialize_reserve()?;
        let signal: u32 = 100;
    }: {
        <pallet::Pallet<T>>::update_reserve(signal, b"Benchmark Update".to_vec())?;
    }
    verify {
        let state = <pallet::ReserveStateStorage<T>>::get();
        let expected = T::BaselineReserve::get().saturating_add(signal.div(10));
        assert_eq!(state.balance, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
