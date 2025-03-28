#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
    register_identity {
        let kyc: Vec<u8> = b"Benchmark KYC Data".to_vec();
    }: {
        <pallet::Pallet<T>>::register_identity(RawOrigin::Signed(account("user", 0, 0)).into(), kyc.clone())?;
    }
    verify {
        let identity = <pallet::Identities<T>>::get(&account("user", 0, 0)).unwrap();
        assert_eq!(identity.kyc_details, kyc);
    }

    update_identity {
        let kyc_initial: Vec<u8> = b"Initial KYC Data".to_vec();
        let kyc_updated: Vec<u8> = b"Updated KYC Data".to_vec();
        <pallet::Pallet<T>>::register_identity(RawOrigin::Signed(account("user", 0, 0)).into(), kyc_initial.clone())?;
    }: {
        <pallet::Pallet<T>>::update_identity(RawOrigin::Signed(account("user", 0, 0)).into(), kyc_updated.clone(), false)?;
    }
    verify {
        let identity = <pallet::Identities<T>>::get(&account("user", 0, 0)).unwrap();
        assert_eq!(identity.kyc_details, kyc_updated);
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
