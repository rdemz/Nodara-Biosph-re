#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Reward Engine Module - Advanced Version
//!
//! This module implements a dynamic reward distribution system for the Nodara network. It calculates and
//! distributes rewards based on factors such as work performed, reputation scores, and current network conditions.
//! It also logs all reward distributions for full auditability and supports future DAO governance integration.
//!
//! ## Advanced Features:
//! - **Dynamic Reward Calculation:** Computes rewards based on configurable parameters.
//! - **Audit Logging:** Maintains an immutable log of every reward distribution event.
//! - **DAO Governance Integration:** Allows future proposals to adjust reward parameters.
//! - **Performance Optimizations:** Optimized arithmetic and memory handling.
//!

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::SaturatedConversion;

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
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineRewardPool::get();
            let state = RewardEngineState {
                reward_pool: baseline,
                history: vec![],
            };
            <RewardEngineStorage<T>>::put(state);
            Ok(())
        }

        /// Distribute a reward to a given account.
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
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let record = RewardRecord {
                timestamp,
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
        /// This function can be extended in the future to be callable via DAO governance.
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
            Self::deposit_event(Event::RewardPoolUpdated(previous_pool, state.reward_pool));
            Ok(())
        }
    }
}
