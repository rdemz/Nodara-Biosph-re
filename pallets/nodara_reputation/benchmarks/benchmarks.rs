#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
    initialize_reputation {
        let caller = account("user", 0, 0);
    }: {
        <pallet::Pallet<T>>::initialize_reputation(RawOrigin::Signed(caller.clone()).into())?;
    }
    verify {
        let state = <pallet::ReputationStateStorage<T>>::get(&caller).unwrap();
        assert_eq!(state.score, T::DefaultReputation::get());
    }

    update_reputation {
        let caller = account("user", 0, 0);
        <pallet::Pallet<T>>::initialize_reputation(RawOrigin::Signed(caller.clone()).into())?;
        let delta: i32 = 20;
        let reason: Vec<u8> = b"Benchmark Positive Update".to_vec();
    }: {
        <pallet::Pallet<T>>::update_reputation(RawOrigin::Signed(caller.clone()).into(), delta, reason.clone())?;
    }
    verify {
        let state = <pallet::ReputationStateStorage<T>>::get(&caller).unwrap();
        let expected = T::DefaultReputation::get().saturating_add(20);
        assert_eq!(state.score, expected);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
