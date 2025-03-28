#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_liquidity {
        // Setup before initializing liquidity
    }: {
        <pallet::Pallet<T>>::initialize_liquidity()?;
    }
    verify {
        let state = <pallet::LiquidityStateStorage<T>>::get();
        assert_eq!(state.current_level, T::BaselineLiquidity::get());
    }

    update_liquidity {
        <pallet::Pallet<T>>::initialize_liquidity()?;
        let signal: u32 = 100;
    }: {
        <pallet::Pallet<T>>::update_liquidity(signal)?;
    }
    verify {
        let state = <pallet::LiquidityStateStorage<T>>::get();
        let expected = T::BaselineLiquidity::get().saturating_add(signal.div(10));
        assert_eq!(state.current_level, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
