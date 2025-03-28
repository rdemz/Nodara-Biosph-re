#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for testing the reserve fund
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
        type BaselineReserve = sp_runtime::traits::ConstU32<1000>;
        type MinReserve = sp_runtime::traits::ConstU32<500>;
        type MaxReserve = sp_runtime::traits::ConstU32<10000>;
        type ReserveSmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_reserve() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reserve());
            let state = <pallet::ReserveStateStorage<TestConfig>>::get();
            assert_eq!(state.balance, 1000);
        });
    }

    #[test]
    fn test_contribute() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reserve());
            let initial_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            let baseline = initial_state.balance;
            assert_ok!(<pallet::Pallet<TestConfig>>::contribute(200, b"Fee Collection".to_vec()));
            let updated_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            assert_eq!(updated_state.balance, baseline + 200);
        });
    }

    #[test]
    fn test_withdraw() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reserve());
            // Contribute additional funds to ensure sufficient balance
            assert_ok!(<pallet::Pallet<TestConfig>>::contribute(300, b"Extra Funds".to_vec()));
            let initial_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            let baseline = initial_state.balance;
            // Withdraw an amount that keeps balance above the minimum (500)
            assert_ok!(<pallet::Pallet<TestConfig>>::withdraw(200, b"Stabilization Withdrawal".to_vec()));
            let updated_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            assert_eq!(updated_state.balance, baseline - 200);
            // Attempt to withdraw an amount that would drop below the minimum should fail
            assert_err!(<pallet::Pallet<TestConfig>>::withdraw(1000, b"Over Withdrawal".to_vec()), pallet::Error::<TestConfig>::InsufficientReserve);
        });
    }

    #[test]
    fn test_update_reserve() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_reserve());
            let initial_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            let baseline = initial_state.balance;
            // Update reserve with a signal of 100, which should add 10 (100/10)
            assert_ok!(<pallet::Pallet<TestConfig>>::update_reserve(100, b"Automatic Adjustment".to_vec()));
            let updated_state = <pallet::ReserveStateStorage<TestConfig>>::get();
            let expected = baseline.saturating_add(100 / 10);
            assert_eq!(updated_state.balance, expected);
        });
    }
}
