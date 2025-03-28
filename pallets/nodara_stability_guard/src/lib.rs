#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_stability_guard - Legendary Edition
//!
//! This module monitors the network’s volatility and dynamically adjusts stability parameters.
//! It uses predictive analytics and a smoothing algorithm to ensure that the network remains stable and resilient,
//! even under fluctuating conditions. Every change is recorded for full auditability and transparency.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing the stability state.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct StabilityState {
    pub parameter: u32,
    pub history: Vec<(u64, u32, u32, u32)>, // (timestamp, previous parameter, new parameter, volatility signal)
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
        /// Baseline stability parameter value.
        #[pallet::constant]
        type BaselineStability: Get<u32>;
        /// Smoothing factor for stability adjustments.
        #[pallet::constant]
        type StabilitySmoothingFactor: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn stability_state)]
    pub type StabilityStateStorage<T: Config> = StorageValue<_, StabilityState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when the stability parameter is updated.
        StabilityUpdated(u32, u32, u32), // (previous parameter, new parameter, volatility signal)
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Provided volatility signal is invalid.
        InvalidVolatilitySignal,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the stability state with the baseline value.
        pub fn initialize_stability() -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineStability::get();
            let state = StabilityState {
                parameter: baseline,
                history: vec![(now, 0, baseline, 0)],
            };
            <StabilityStateStorage<T>>::put(state);
            Ok(())
        }

        /// Updates the stability parameter based on a given volatility signal.
        ///
        /// The new stability parameter is calculated using a smoothing algorithm:
        ///   new_parameter = current_parameter + (volatility_signal / smoothing_factor)
        pub fn update_stability(volatility_signal: u32) -> DispatchResult {
            ensure!(volatility_signal > 0, Error::<T>::InvalidVolatilitySignal);
            let mut state = <StabilityStateStorage<T>>::get();
            let previous = state.parameter;
            let adjustment = volatility_signal / T::StabilitySmoothingFactor::get();
            let new_parameter = previous.saturating_add(adjustment);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push((now, previous, new_parameter, volatility_signal));
            state.parameter = new_parameter;
            <StabilityStateStorage<T>>::put(state);
            Self::deposit_event(Event::StabilityUpdated(previous, new_parameter, volatility_signal));
            Ok(())
        }
    }
}
