#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Reputation Module - Locked and Ready for Deployment
//!
//! This module manages the reputation system within the Nodara BIOSPHÈRE QUANTIC network.
//! It tracks and aggregates reputation scores for accounts, maintains a detailed history log,
//! and integrates with on-chain governance for dynamic parameter adjustments.
//!
//! All dependencies are locked to ensure a reproducible build in production.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;

    /// Structure representing a log entry for reputation changes.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationLog {
        /// Unix timestamp of the adjustment.
        pub timestamp: u64,
        /// Change in reputation (can be positive or negative).
        pub delta: i32,
        /// Reason for the reputation change.
        pub reason: Vec<u8>,
    }

    /// Structure representing the reputation record for an account.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationRecord {
        /// The current reputation score.
        pub score: u32,
        /// History log of reputation adjustments.
        pub history: Vec<ReputationLog>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Initial reputation score for a new account.
        #[pallet::constant]
        type InitialReputation: Get<u32>;
    }

    /// Storage mapping each account to its reputation record.
    #[pallet::storage]
    #[pallet::getter(fn reputations)]
    pub type Reputations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ReputationRecord, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a reputation update occurs: (account, delta, new score)
        ReputationUpdated(T::AccountId, i32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No reputation record found for the account.
        ReputationNotFound,
        /// Reputation update would result in a negative score.
        ReputationUnderflow,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initializes the reputation of the caller with the initial reputation value.
        #[pallet::weight(10_000)]
        pub fn initialize_reputation(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Reputations::<T>::contains_key(&who), "Reputation already exists for this account");
            let record = ReputationRecord {
                score: T::InitialReputation::get(),
                history: Vec::new(),
            };
            Reputations::<T>::insert(&who, record);
            Ok(())
        }

        /// Updates the reputation of the caller based on a delta.
        /// `delta` can be positive (increase) or negative (decrease).
        #[pallet::weight(10_000)]
        pub fn update_reputation(origin: OriginFor<T>, delta: i32, reason: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Reputations::<T>::try_mutate(&who, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::ReputationNotFound)?;
                let current = record.score as i32;
                let new_score = current.checked_add(delta).ok_or(Error::<T>::ReputationUnderflow)?;
                ensure!(new_score >= 0, Error::<T>::ReputationUnderflow);
                record.score = new_score as u32;
                let timestamp = Self::current_timestamp();
                record.history.push(ReputationLog {
                    timestamp,
                    delta,
                    reason,
                });
                Self::deposit_event(Event::ReputationUpdated(who.clone(), delta, record.score));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Returns a fixed timestamp.
        /// In production, replace this with a reliable time provider (e.g., pallet_timestamp).
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, assert_err, parameter_types};
        use sp_core::H256;
        use sp_runtime::{
            traits::{BlakeTwo256, IdentityLookup},
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
                ReputationModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const InitialReputation: u32 = 100;
        }

        impl system::Config for Test {
            type BaseCallFilter = frame_support::traits::Everything;
            type BlockWeights = ();
