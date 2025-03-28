#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_id
    pub struct TestConfig;
    impl frame_system::Config for TestConfig {
        type BaseCallFilter = ();
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = ();
        type RuntimeCall = ();
        type BlockNumber = u64;
        type Hash = sp_core::H256;
        type Hashing = sp_core::H256;
        type AccountId = u64;
        type Lookup = ();
        type Header = ();
        type Index = u64;
        type BlockHashCount = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type RuntimeEvent = ();
        type Version = ();
        type PalletInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = ();
    }

    impl pallet::Config for TestConfig {
        type RuntimeEvent = ();
        type MaxKycLength = sp_runtime::traits::ConstU32<256>;
        type DefaultVerification = sp_runtime::traits::ConstBool<true>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_register_identity() {
        new_test_ext().execute_with(|| {
            let kyc = b"Sample KYC Data".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::register_identity(RawOrigin::Signed(1).into(), kyc.clone()));
            let identity = <pallet::Identities<TestConfig>>::get(&1).unwrap();
            assert_eq!(identity.kyc_details, kyc);
            assert_eq!(identity.verified, true);
        });
    }

    #[test]
    fn test_update_identity() {
        new_test_ext().execute_with(|| {
            let initial_kyc = b"Initial KYC Data".to_vec();
            let updated_kyc = b"Updated KYC Data".to_vec();
            // First, register identity
            assert_ok!(<pallet::Pallet<TestConfig>>::register_identity(RawOrigin::Signed(1).into(), initial_kyc.clone()));
            // Then, update identity
            assert_ok!(<pallet::Pallet<TestConfig>>::update_identity(RawOrigin::Signed(1).into(), updated_kyc.clone(), false));
            let identity = <pallet::Identities<TestConfig>>::get(&1).unwrap();
            assert_eq!(identity.kyc_details, updated_kyc);
            assert_eq!(identity.verified, false);
        });
    }
}
