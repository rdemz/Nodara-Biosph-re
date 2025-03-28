#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Create a mock configuration for testing purposes
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
        type BaselineLiquidity = sp_runtime::traits::ConstU32<1000>;
        type LiquiditySmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_liquidity() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_liquidity());
            let state = <pallet::LiquidityStateStorage<TestConfig>>::get();
            assert_eq!(state.current_level, 1000);
        });
    }

    #[test]
    fn test_update_liquidity() {
        new_test_ext().execute_with(|| {
            // Initialize first
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_liquidity());
            let initial_state = <pallet::LiquidityStateStorage<TestConfig>>::get();
            let baseline = initial_state.current_level;
            // Update liquidity with a signal of 100
            assert_ok!(<pallet::Pallet<TestConfig>>::update_liquidity(100));
            let updated_state = <pallet::LiquidityStateStorage<TestConfig>>::get();
            let expected = baseline.saturating_add(100 / 10);
            assert_eq!(updated_state.current_level, expected);
        });
    }
}
