#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_growth_model - Legendary Edition
//!
//! This module dynamically adjusts the reward multiplier for Nodara BIOSPHÈRE QUANTIC based on real-time network signals.
//! It uses a smoothing algorithm to ensure gradual changes and logs each update for complete traceability.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing the growth state.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct GrowthState {
    pub multiplier: u32,
    pub history: Vec<(u64, u32, u32)>, // (timestamp, previous multiplier, new multiplier, signal)
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
        /// Baseline multiplier value.
        #[pallet::constant]
        type BaselineMultiplier: Get<u32>;
        /// Smoothing factor to reduce abrupt changes.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn growth_state)]
    pub type GrowthStateStorage<T: Config> = StorageValue<_, GrowthState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when the growth multiplier is updated.
        GrowthMultiplierUpdated(u32, u32, u32), // (previous multiplier, new multiplier, signal)
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Signal provided is invalid.
        InvalidSignal,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the growth state with the baseline multiplier.
        pub fn initialize_growth() -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineMultiplier::get();
            let state = GrowthState {
                multiplier: baseline,
                history: vec![(now, 0, baseline)],
            };
            <GrowthStateStorage<T>>::put(state);
            Ok(())
        }

        /// Updates the growth multiplier based on an incoming signal.
        ///
        /// The new multiplier is calculated as:
        ///     new_multiplier = current_multiplier + (signal / smoothing_factor)
        pub fn update_multiplier(signal: u32) -> DispatchResult {
            ensure!(signal > 0, Error::<T>::InvalidSignal);
            let mut state = <GrowthStateStorage<T>>::get();
            let previous = state.multiplier;
            let adjustment = signal / T::SmoothingFactor::get();
            let new_multiplier = previous.saturating_add(adjustment);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push((now, previous, new_multiplier));
            state.multiplier = new_multiplier;
            <GrowthStateStorage<T>>::put(state);
            Self::deposit_event(Event::GrowthMultiplierUpdated(previous, new_multiplier, signal));
            Ok(())
        }
    }
}
