#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_reputation - Legendary Edition
//!
//! This module implements a decentralized reputation system for Nodara BIOSPHÈRE QUANTIC.
//! It assigns and updates reputation scores for network participants based on their activity, performance,
//! and contributions. All updates are logged immutably for full transparency and auditability.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure for storing reputation scores.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ReputationState {
    pub score: u32,
    pub history: Vec<(u64, u32, u32, Vec<u8>)>, // (timestamp, previous score, new score, reason)
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
        /// Default reputation score for new accounts.
        #[pallet::constant]
        type DefaultReputation: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn reputation_state)]
    pub type ReputationStateStorage<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ReputationState, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a reputation score is updated (account, previous score, new score, reason).
        ReputationUpdated(T::AccountId, u32, u32, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The provided reputation update is invalid.
        InvalidReputationUpdate,
        /// No reputation state found for the account.
        ReputationNotFound,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the reputation state for a new account.
        pub fn initialize_reputation(origin: T::Origin) -> DispatchResult {
            let account = ensure_signed(origin)?;
            ensure!(!ReputationStateStorage::<T>::contains_key(&account), Error::<T>::InvalidReputationUpdate);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let state = ReputationState {
                score: T::DefaultReputation::get(),
                history: vec![(now, 0, T::DefaultReputation::get(), b"Initialization".to_vec())],
            };
            ReputationStateStorage::<T>::insert(account.clone(), state);
            Self::deposit_event(Event::ReputationUpdated(account, 0, T::DefaultReputation::get(), b"Initialization".to_vec()));
            Ok(())
        }

        /// Updates the reputation score for an account.
        ///
        /// The reputation is updated by a delta (which can be positive or negative). The function logs the change.
        pub fn update_reputation(origin: T::Origin, delta: i32, reason: Vec<u8>) -> DispatchResult {
            let account = ensure_signed(origin)?;
            ReputationStateStorage::<T>::try_mutate(&account, |maybe_state| -> DispatchResult {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ReputationNotFound)?;
                let previous = state.score;
                // Calculate new score safely, ensuring it doesn't underflow
                let new_score = if delta < 0 {
                    previous.saturating_sub(delta.wrapping_abs() as u32)
                } else {
                    previous.saturating_add(delta as u32)
                };
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                state.history.push((now, previous, new_score, reason.clone()));
                state.score = new_score;
                Ok(())
            })?;
            Self::deposit_event(Event::ReputationUpdated(account, 0, ReputationStateStorage::<T>::get(&account).unwrap().score, reason));
            Ok(())
        }
    }
}
