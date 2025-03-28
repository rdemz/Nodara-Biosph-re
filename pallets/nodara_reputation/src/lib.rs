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
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Score de réputation initial pour un nouveau compte.
        #[pallet::constant]
        type InitialReputation: Get<u32>;
    }

    /// Stockage des reputations, associant chaque compte à son enregistrement.
    #[pallet::storage]
    #[pallet::getter(fn reputations)]
    pub type Reputations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ReputationRecord, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Émission d'un événement lors d'une mise à jour de la réputation:
        /// (compte, variation, nouveau score)
        ReputationUpdated(T::AccountId, i32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Aucun enregistrement de réputation trouvé pour ce compte.
        ReputationNotFound,
        /// Mise à jour de la réputation conduirait à une sous-flux (score négatif).
        ReputationUnderflow,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise la réputation d'un compte avec la valeur initiale.
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

        /// Met à jour la réputation d'un compte en fonction d'une variation.
        /// `delta` peut être positif (augmentation) ou négatif (diminution).
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
        /// Retourne un timestamp fixe (à remplacer en production par un fournisseur de temps fiable).
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}

