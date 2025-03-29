#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

/// # Nodara Reward Engine Module - Advanced Version
///
/// This module implements a dynamic reward distribution system for the Nodara network. It calculates and
/// distributes rewards based on configurable parameters such as work performed and reputation scores.
/// All reward distributions are logged for auditability and the module is designed for future DAO governance integration.
///
/// ## Advanced Features:
/// - **Dynamic Reward Calculation:** Computes rewards based on configurable parameters.
/// - **Audit Logging:** Maintains an immutable log of every reward distribution event.
/// - **DAO Governance Integration:** Allows future proposals to adjust reward parameters.
/// - **Performance Optimizations:** Optimized arithmetic and memory handling.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;

    /// Structure representing a reward distribution record.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct RewardRecord<AccountId> {
        pub timestamp: u64,
        pub account: AccountId,
        pub reward_amount: u128,
        pub details: Vec<u8>,
    }

    /// Global state of the reward engine.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct RewardEngineState<AccountId> {
        pub reward_pool: u128,
        pub history: Vec<RewardRecord<AccountId>>,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration for the Reward Engine module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Baseline reward pool for initialization.
        #[pallet::constant]
        type BaselineRewardPool: Get<u128>;
    }

    /// Storage for the reward engine state.
    #[pallet::storage]
    #[pallet::getter(fn reward_engine_state)]
    pub type RewardEngineStorage<T: Config> =
        StorageValue<_, RewardEngineState<T::AccountId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a reward is distributed (account, reward amount, details).
        RewardDistributed(T::AccountId, u128, Vec<u8>),
        /// Emitted when the reward pool is updated (previous pool, new pool).
        RewardPoolUpdated(u128, u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient funds in the reward pool.
        InsufficientRewardPool,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialize the reward engine with a baseline reward pool.
        /// Can only be called by Root.
        #[pallet::weight(10_000)]
        pub fn initialize_rewards(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp_now = <timestamp::Pallet<T>>::get();
            let baseline = T::BaselineRewardPool::get();
            let state = RewardEngineState {
                reward_pool: baseline,
                history: vec![],
            };
            <RewardEngineStorage<T>>::put(state);
            // You may emit an event here if needed.
            Ok(())
        }

        /// Distribute a reward to a given account.
        ///
        /// The reward is subtracted from the reward pool and logged.
        #[pallet::weight(10_000)]
        pub fn distribute_reward(
            origin: OriginFor<T>,
            account: T::AccountId,
            reward: u128,
            details: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <RewardEngineStorage<T>>::get();
            ensure!(state.reward_pool >= reward, Error::<T>::InsufficientRewardPool);
            let previous_pool = state.reward_pool;
            state.reward_pool = state.reward_pool.saturating_sub(reward);
            let now = <timestamp::Pallet<T>>::get();
            let record = RewardRecord {
                timestamp: now,
                account: account.clone(),
                reward_amount: reward,
                details: details.clone(),
            };
            state.history.push(record);
            <RewardEngineStorage<T>>::put(state);
            Self::deposit_event(Event::RewardDistributed(account, reward, details));
            Self::deposit_event(Event::RewardPoolUpdated(previous_pool, previous_pool.saturating_sub(reward)));
            Ok(())
        }

        /// Update the reward pool by a given amount.
        ///
        /// If `increase` is true, the amount is added; otherwise, it is subtracted.
        #[pallet::weight(10_000)]
        pub fn update_reward_pool(origin: OriginFor<T>, amount: u128, increase: bool) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <RewardEngineStorage<T>>::get();
            let previous_pool = state.reward_pool;
            if increase {
                state.reward_pool = state.reward_pool.saturating_add(amount);
            } else {
                ensure!(state.reward_pool >= amount, Error::<T>::InsufficientRewardPool);
                state.reward_pool = state.reward_pool.saturating_sub(amount);
            }
            <RewardEngineStorage<T>>::put(state);
            Self::deposit_event(Event::RewardPoolUpdated(previous_pool, <RewardEngineStorage<T>>::get().reward_pool));
            Ok(())
        }

        /// Distribute a dynamic reward calculated from input parameters.
        ///
        /// For example, reward can be computed based on work performed and reputation.
        /// This extrinsic computes the reward using `calculate_dynamic_reward` and then distributes it.
        #[pallet::weight(10_000)]
        pub fn distribute_dynamic_reward(
            origin: OriginFor<T>,
            account: T::AccountId,
            work: u128,
            reputation: u128,
            details: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            // Calculate dynamic reward based on work and reputation.
            let reward = Self::calculate_dynamic_reward(work, reputation);
            // Reuse distribute_reward logic.
            Self::distribute_reward(origin, account, reward, details)
        }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate dynamic reward based on input factors.
        ///
        /// This is a simple example formula:
        /// reward = work * reputation_factor, where reputation_factor is derived from reputation.
        /// The formula can be refined as needed.
        fn calculate_dynamic_reward(work: u128, reputation: u128) -> u128 {
            // For illustration, let’s assume reputation_factor is:
            // reputation_factor = 1 + (reputation / 1000)
            let reputation_factor = 1u128.saturating_add(reputation / 1_000);
            work.saturating_mul(reputation_factor)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, assert_err, parameter_types};
        use sp_core::H256;
        use sp_runtime::{
            traits::{BlakeTwo256, IdentityLookup, Saturating},
            testing::Header,
        };
        use frame_system as system;

        type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
        type Block = system::mocking::MockBlock<Test>;

        frame_support::construct_runtime!(
            pub enum Test where
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                RewardEngineModule: Pallet,
                Timestamp: timestamp::Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineRewardPool: u128 = 1_000_000;
            pub const MinimumPeriod: u64 = 1;
        }

        impl system::Config for Test {
            type BaseCallFilter = frame_support::traits::Everything;
            type BlockWeights = ();
            type BlockLength = ();
            type DbWeight = ();
            type RuntimeOrigin = system::mocking::Origin;
            type RuntimeCall = Call;
            type Index = u64;
            type BlockNumber = u64;
            type Hash = H256;
            type Hashing = BlakeTwo256;
            type AccountId = u64;
            type Lookup = IdentityLookup<Self::AccountId>;
            type Header = Header;
            type RuntimeEvent = ();
            type BlockHashCount = BlockHashCount;
            type Version = ();
            type PalletInfo = ();
            type AccountData = ();
            type OnNewAccount = ();
            type OnKilledAccount = ();
            type SystemWeightInfo = ();
            type SS58Prefix = ();
            type OnSetCode = ();
            type MaxConsumers = ();
        }

        impl timestamp::Config for Test {
            type Moment = u64;
            type OnTimestampSet = ();
            type MinimumPeriod = MinimumPeriod;
            type WeightInfo = ();
        }

        impl Config for Test {
            type RuntimeEvent = ();
            type BaselineRewardPool = BaselineRewardPool;
        }

        #[test]
        fn initialize_rewards_works() {
            assert_ok!(RewardEngineModule::initialize_rewards(system::RawOrigin::Root.into()));
            let state = RewardEngineModule::reward_engine_state();
            assert_eq!(state.reward_pool, BaselineRewardPool::get());
            assert!(state.history.is_empty());
        }

        #[test]
        fn distribute_reward_works() {
            let account = 1;
            // Initialize the reward engine.
            assert_ok!(RewardEngineModule::initialize_rewards(system::RawOrigin::Root.into()));
            // Distribute a reward.
            let reward = 100_000;
            let details = b"Test reward".to_vec();
            assert_ok!(RewardEngineModule::distribute_reward(system::RawOrigin::Signed(2).into(), account, reward, details.clone()));
            let state = RewardEngineModule::reward_engine_state();
            assert_eq!(state.reward_pool, BaselineRewardPool::get() - reward);
            assert!(!state.history.is_empty());
        }

        #[test]
        fn distribute_dynamic_reward_works() {
            let account = 1;
            // Initialize the reward engine.
            assert_ok!(RewardEngineModule::initialize_rewards(system::RawOrigin::Root.into()));
            // Assume work=200,000 and reputation=5,000.
            let work = 200_000;
            let reputation = 5_000;
            // Expected dynamic reward: 200,000 * (1 + 5000/1000) = 200,000 * 6 = 1,200,000.
            // But reward pool is limited, so distribution should fail if pool insufficient.
            assert_err!(
                RewardEngineModule::distribute_dynamic_reward(system::RawOrigin::Signed(2).into(), account, work, reputation, b"Dynamic".to_vec()),
                Error::<Test>::InsufficientRewardPool
            );
            // Increase reward pool.
            assert_ok!(RewardEngineModule::update_reward_pool(system::RawOrigin::Signed(2).into(), 1_500_000, true));
            // Now distribution should work.
            assert_ok!(RewardEngineModule::distribute_dynamic_reward(system::RawOrigin::Signed(2).into(), account, work, reputation, b"Dynamic".to_vec()));
        }

        #[test]
        fn update_reward_pool_works() {
            assert_ok!(RewardEngineModule::initialize_rewards(system::RawOrigin::Root.into()));
            let current_pool = RewardEngineModule::reward_engine_state().reward_pool;
            // Increase pool.
            let increase_amount = 200_000;
            assert_ok!(RewardEngineModule::update_reward_pool(system::RawOrigin::Signed(2).into(), increase_amount, true));
            let new_pool = RewardEngineModule::reward_engine_state().reward_pool;
            assert_eq!(new_pool, current_pool + increase_amount);
            // Decrease pool.
            let decrease_amount = 100_000;
            assert_ok!(RewardEngineModule::update_reward_pool(system::RawOrigin::Signed(2).into(), decrease_amount, false));
            let final_pool = RewardEngineModule::reward_engine_state().reward_pool;
            assert_eq!(final_pool, current_pool + increase_amount - decrease_amount);
        }
    }
}
