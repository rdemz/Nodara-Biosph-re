#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_reward_engine - Legendary Edition
//!
//! This module calculates and distributes rewards within Nodara BIOSPHÈRE QUANTIC. It takes into account
//! metrics such as work performed, reputation scores, and network conditions to compute dynamic rewards.
//! The module ensures that reward distributions are fair, transparent, and adaptive to real-time changes.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing the state of the reward engine.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct RewardState {
    pub reward_pool: u32,
    pub history: Vec<(u64, u32, u32, Vec<u8>)>, // (timestamp, previous pool, new pool, details)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Initial reward pool balance.
        #[pallet::constant]
        type BaselineRewardPool: Get<u32>;
        /// Smoothing factor for reward pool adjustments.
        #[pallet::constant]
        type RewardSmoothingFactor: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn reward_state)]
    pub type RewardStateStorage<T: Config> = StorageValue<_, RewardState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when rewards are distributed (account, amount, work_metric, reputation).
        RewardDistributed(T::AccountId, u32, u32, u32),
        /// Emitted when the reward pool is updated (previous pool, new pool, adjustment signal).
        RewardPoolUpdated(u32, u32, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient funds in the reward pool.
        InsufficientRewardPool,
        /// Invalid reward calculation parameters.
        InvalidRewardParameters,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the reward engine with the baseline reward pool.
        pub fn initialize_rewards() -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineRewardPool::get();
            let state = RewardState {
                reward_pool: baseline,
                history: vec![(now, 0, baseline, b"Initialization".to_vec())],
            };
            <RewardStateStorage<T>>::put(state);
            Ok(())
        }

        /// Distributes rewards to a specified account based on work performed and reputation.
        ///
        /// Parameters:
        /// - `account`: The account receiving the reward.
        /// - `work_metric`: A numeric indicator of the work performed.
        /// - `reputation`: A numeric reputation score for the account.
        ///
        /// The reward amount is calculated as:
        ///   reward = (work_metric * reputation) / some_factor (for simplicity, factor is hardcoded here)
        pub fn distribute_reward(account: T::AccountId, work_metric: u32, reputation: u32) -> DispatchResult {
            let factor: u32 = 10; // Example factor for reward calculation
            let reward = work_metric.saturating_mul(reputation) / factor;
            let mut state = <RewardStateStorage<T>>::get();
            ensure!(state.reward_pool >= reward, Error::<T>::InsufficientRewardPool);
            let previous_pool = state.reward_pool;
            state.reward_pool = state.reward_pool.saturating_sub(reward);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push((now, previous_pool, state.reward_pool, b"Reward Distribution".to_vec()));
            <RewardStateStorage<T>>::put(state);
            Self::deposit_event(Event::RewardDistributed(account, reward, work_metric, reputation));
            Ok(())
        }

        /// Automatically updates the reward pool based on an economic adjustment signal.
        ///
        /// The new reward pool is calculated as:
        ///   new_pool = current_pool + (signal / RewardSmoothingFactor)
        pub fn update_reward_pool(signal: u32, details: Vec<u8>) -> DispatchResult {
            ensure!(signal > 0, Error::<T>::InvalidRewardParameters);
            <RewardStateStorage<T>>::mutate(|state| {
                let previous = state.reward_pool;
                let adjustment = signal / T::RewardSmoothingFactor::get();
                state.reward_pool = state.reward_pool.saturating_add(adjustment);
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                state.history.push((now, previous, state.reward_pool, details.clone()));
            });
            Self::deposit_event(Event::RewardPoolUpdated(
                <RewardStateStorage<T>>::get().history.last().unwrap().1,
                <RewardStateStorage<T>>::get().reward_pool,
                details,
            ));
            Ok(())
        }
    }
}
