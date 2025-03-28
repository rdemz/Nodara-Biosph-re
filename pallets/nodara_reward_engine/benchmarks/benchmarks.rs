#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    initialize_rewards {
        // Setup for initialization benchmark
    }: {
        <pallet::Pallet<T>>::initialize_rewards()?;
    }
    verify {
        let state = <pallet::RewardStateStorage<T>>::get();
        assert_eq!(state.reward_pool, T::BaselineRewardPool::get());
    }

    distribute_reward {
        <pallet::Pallet<T>>::initialize_rewards()?;
        let work_metric: u32 = 50;
        let reputation: u32 = 20;
    }: {
        <pallet::Pallet<T>>::distribute_reward(account("recipient", 0, 0), work_metric, reputation)?;
    }
    verify {
        let state = <pallet::RewardStateStorage<T>>::get();
        let expected = T::BaselineRewardPool::get().saturating_sub(50 * 20 / 10);
        assert_eq!(state.reward_pool, expected);
    }

    update_reward_pool {
        <pallet::Pallet<T>>::initialize_rewards()?;
        let signal: u32 = 100;
    }: {
        <pallet::Pallet<T>>::update_reward_pool(signal, b"Benchmark Update".to_vec())?;
    }
    verify {
        let state = <pallet::RewardStateStorage<T>>::get();
        let expected = T::BaselineRewardPool::get().saturating_add(signal.div(10));
        assert_eq!(state.reward_pool, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
