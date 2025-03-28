#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_pow
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
        type BaselineDifficulty = sp_runtime::traits::ConstU32<100>;
        type PowSmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_pow() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_pow());
            let state = <pallet::PowStateStorage<TestConfig>>::get();
            assert_eq!(state.difficulty, 100);
        });
    }

    #[test]
    fn test_submit_work_valid() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_pow());
            // Submit work with value equal to difficulty (100)
            assert_ok!(<pallet::Pallet<TestConfig>>::submit_work(RawOrigin::Signed(1).into(), 100, b"Signature".to_vec()));
        });
    }

    #[test]
    fn test_submit_work_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_pow());
            // Submit work with a value lower than difficulty should be rejected
            assert_err!(<pallet::Pallet<TestConfig>>::submit_work(RawOrigin::Signed(1).into(), 50, b"Signature".to_vec()), pallet::Error::<TestConfig>::WorkRejected);
        });
    }

    #[test]
    fn test_adjust_difficulty() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_pow());
            let initial_state = <pallet::PowStateStorage<TestConfig>>::get();
            let baseline = initial_state.difficulty;
            // Adjust difficulty with a signal of 50, which should add 5 (50/10)
            assert_ok!(<pallet::Pallet<TestConfig>>::adjust_difficulty(50));
            let updated_state = <pallet::PowStateStorage<TestConfig>>::get();
            let expected = baseline.saturating_add(50 / 10);
            assert_eq!(updated_state.difficulty, expected);
        });
    }
}
