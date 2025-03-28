#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
    submit_iot_data {
        let payload: Vec<u8> = b"Benchmark IoT Data".to_vec();
        let device_id: Vec<u8> = b"BenchmarkDevice".to_vec();
        let signature: Vec<u8> = b"BenchmarkSignature".to_vec();
    }: {
        <pallet::Pallet<T>>::submit_iot_data(1, payload.clone(), device_id.clone(), signature.clone())?;
    }
    verify {
        let record = <pallet::IotData<T>>::get(1).unwrap();
        assert_eq!(record.payload, payload);
    }

    update_config {
        let new_config: Vec<u8> = b"BenchmarkConfig".to_vec();
        let details: Vec<u8> = b"Benchmark details".to_vec();
    }: {
        <pallet::Pallet<T>>::update_config(new_config.clone(), details.clone())?;
    }
    verify {
        // Verification is based on event emission and mock log entries.
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
