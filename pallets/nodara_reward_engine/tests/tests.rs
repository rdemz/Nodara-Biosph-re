#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for testing the reward engine
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
        type BaselineRewardPool = sp_runtime::traits::ConstU32<1000>;
        type RewardSmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_initialize_rewards() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_rewards());
            let state = <pallet::RewardStateStorage<TestConfig>>::get();
            assert_eq!(state.reward_pool, 1000);
        });
    }

    #[test]
    fn test_distribute_reward() {
        new_test_ext().execute_with(|| {
            // Initialize rewards
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_rewards());
            let initial_state = <pallet::RewardStateStorage<TestConfig>>::get();
            let baseline = initial_state.reward_pool;
            // Distribute reward with work_metric=50 and reputation=20
            assert_ok!(<pallet::Pallet<TestConfig>>::distribute_reward(1, 50, 20));
            let updated_state = <pallet::RewardStateStorage<TestConfig>>::get();
            let expected_reward = 50 * 20 / 10; // factor=10
            assert_eq!(updated_state.reward_pool, baseline.saturating_sub(expected_reward));
        });
    }

    #[test]
    fn test_update_reward_pool() {
        new_test_ext().execute_with(|| {
            assert_ok!(<pallet::Pallet<TestConfig>>::initialize_rewards());
            let initial_state = <pallet::RewardStateStorage<TestConfig>>::get();
            let baseline = initial_state.reward_pool;
            // Update reward pool with a signal of 100, which should add 10 (100/10)
            assert_ok!(<pallet::Pallet<TestConfig>>::update_reward_pool(100, b"Test Update".to_vec()));
            let updated_state = <pallet::RewardStateStorage<TestConfig>>::get();
            let expected = baseline.saturating_add(100 / 10);
            assert_eq!(updated_state.reward_pool, expected);
        });
    }
}
