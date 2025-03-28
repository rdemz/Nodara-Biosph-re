#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_reputation
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
        type DefaultReputation = sp_runtime::traits::ConstU32<100>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_reputation() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reputation(RawOrigin::Signed(1).into()));
            let state = <pallet::ReputationStateStorage<TestConfig>>::get(&1).unwrap();
            assert_eq!(state.score, 100);
        });
    }

    #[test]
    fn test_update_reputation_positive() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reputation(RawOrigin::Signed(1).into()));
            // Update reputation by +20
            assert_ok!(<pallet::Pallet<TestConfig>>::update_reputation(RawOrigin::Signed(1).into(), 20, b"Positive Contribution".to_vec()));
            let state = <pallet::ReputationStateStorage<TestConfig>>::get(&1).unwrap();
            assert_eq!(state.score, 120);
        });
    }

    #[test]
    fn test_update_reputation_negative() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reputation(RawOrigin::Signed(1).into()));
            // Update reputation by -30, should not underflow
            assert_ok!(<pallet::Pallet<TestConfig>>::update_reputation(RawOrigin::Signed(1).into(), -30, b"Negative Feedback".to_vec()));
            let state = <pallet::ReputationStateStorage<TestConfig>>::get(&1).unwrap();
            // Expected new score is 70 (100 - 30)
            assert_eq!(state.score, 70);
        });
    }
}
