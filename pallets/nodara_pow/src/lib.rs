#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_pow - Legendary Edition
//!
//! This module implements a dynamic, biomimetic Proof-of-Work (PoW) mechanism for Nodara BIOSPHÈRE QUANTIC.
//! It securely validates work submissions from miners, dynamically adjusts mining difficulty based on network conditions,
//! and logs all operations for full auditability.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::{RuntimeDebug, traits::SaturatedConversion};
use parity_scale_codec::{Encode, Decode};

/// Structure representing the PoW state.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct PowState {
    pub difficulty: u32,
    pub total_work: u32,
    pub history: Vec<(u64, u32, u32, u32)>, // (timestamp, previous difficulty, new difficulty, work submitted)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Baseline mining difficulty.
        #[pallet::constant]
        type BaselineDifficulty: Get<u32>;
        /// Smoothing factor for difficulty adjustments.
        #[pallet::constant]
        type PowSmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de PoW.
    #[pallet::storage]
    #[pallet::getter(fn pow_state)]
    pub type PowStateStorage<T: Config> = StorageValue<_, PowState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a work submission is validated and recorded.
        PowSubmitted(T::AccountId, u32),
        /// Emitted when the mining difficulty is adjusted (previous, new, work submitted).
        DifficultyAdjusted(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Work submission does not meet the required difficulty.
        WorkRejected,
        /// Invalid work submission.
        InvalidWork,
    }

    /// Dispatchable functions for the module.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initializes the PoW state with a baseline difficulty.
        ///
        /// Can only be called by Root.
        #[pallet::weight(10_000)]
        pub fn initialize_pow(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineDifficulty::get();
            let state = PowState {
                difficulty: baseline,
                total_work: 0,
                history: vec![(now, 0, baseline, 0)],
            };
            <PowStateStorage<T>>::put(state);
            Ok(())
        }

        /// Submits mining work. Validates that the submitted work meets the current difficulty.
        ///
        /// For demonstration purposes, the work submission is simplified.
        /// In a real implementation, work would involve solving a cryptographic puzzle.
        #[pallet::weight(10_000)]
        pub fn submit_work(
            origin: OriginFor<T>,
            work_value: u32,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            ensure!(work_value > 0, Error::<T>::InvalidWork);
            // Simulated work verification using the signature.
            ensure!(!signature.is_empty(), Error::<T>::InvalidWork);
            let state = <PowStateStorage<T>>::get();
            // Work is accepted if work_value meets the current difficulty.
            ensure!(work_value >= state.difficulty, Error::<T>::WorkRejected);

            // Update total work.
            <PowStateStorage<T>>::mutate(|s| {
                s.total_work = s.total_work.saturating_add(work_value);
            });

            Self::deposit_event(Event::PowSubmitted(miner, work_value));
            Ok(())
        }

        /// Adjusts the mining difficulty based on an input signal.
        ///
        /// The new difficulty is calculated as:
        ///   new_difficulty = current_difficulty + (signal / PowSmoothingFactor)
        #[pallet::weight(10_000)]
        pub fn adjust_difficulty(
            origin: OriginFor<T>,
            signal: u32,
        ) -> DispatchResult {
            // Requirement: only a signed call can trigger an adjustment.
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidWork);
            <PowStateStorage<T>>::mutate(|s| {
                let previous = s.difficulty;
                let adjustment = signal / T::PowSmoothingFactor::get();
                let new_difficulty = previous.saturating_add(adjustment);
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                s.history.push((now, previous, new_difficulty, signal));
                s.difficulty = new_difficulty;
            });
            let state = <PowStateStorage<T>>::get();
            // Retrieve previous difficulty from the last history record.
            let last_record = state.history.last().unwrap();
            Self::deposit_event(Event::DifficultyAdjusted(last_record.1, state.difficulty, signal));
            Ok(())
        }
    }
}
