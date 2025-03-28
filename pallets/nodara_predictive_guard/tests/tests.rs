#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_predictive_guard
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
        type BaselineParameter = sp_runtime::traits::ConstU32<100>;
        type PredictionSmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_prediction() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_prediction());
            let state = <pallet::PredictiveStateStorage<TestConfig>>::get();
            assert_eq!(state.parameter, 100);
        });
    }

    #[test]
    fn test_analyze_and_predict() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_prediction());
            let initial_state = <pallet::PredictiveStateStorage<TestConfig>>::get();
            let baseline = initial_state.parameter;
            // Update with a signal of 50
            assert_ok!(<pallet::Pallet<TestConfig>>::analyze_and_predict(50));
            let updated_state = <pallet::PredictiveStateStorage<TestConfig>>::get();
            let expected = baseline.saturating_add(50 / 10);
            assert_eq!(updated_state.parameter, expected);
        });
    }
}
