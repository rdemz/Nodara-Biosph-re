#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
    send_message {
        let payload: Vec<u8> = b"Benchmark Interop Message".to_vec();
        let signature: Vec<u8> = b"BenchmarkSignature".to_vec();
    }: {
        <pallet::Pallet<T>>::send_message(1, payload.clone(), signature.clone())?;
    }
    verify {
        let message = <pallet::OutgoingMessages<T>>::get(1).unwrap();
        assert_eq!(message.payload, payload);
    }

    receive_message {
        let payload: Vec<u8> = b"Benchmark Incoming Message".to_vec();
        let signature: Vec<u8> = b"BenchmarkSignature".to_vec();
    }: {
        <pallet::Pallet<T>>::receive_message(2, payload.clone(), signature.clone())?;
    }
    verify {
        let message = <pallet::IncomingMessages<T>>::get(2).unwrap();
        assert_eq!(message.payload, payload);
    }

    update_config {
        let new_config: Vec<u8> = b"Benchmark Config".to_vec();
        let details: Vec<u8> = b"Benchmark Details".to_vec();
    }: {
        <pallet::Pallet<T>>::update_config(new_config.clone(), details.clone())?;
    }
    verify {
        // Verification is based on event logging and mock history entries.
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
