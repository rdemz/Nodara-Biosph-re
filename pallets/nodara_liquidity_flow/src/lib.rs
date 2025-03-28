#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_liquidity_flow - Legendary Edition
//!
//! This module manages the dynamic liquidity of Nodara BIOSPHÈRE QUANTIC by monitoring current liquidity levels
//! and automatically redistributing funds to maintain optimal network performance. It utilizes a smoothing algorithm
//! to calculate adjustments based on incoming signals and logs every change for full transparency.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing the liquidity state.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct LiquidityState {
    pub current_level: u32,
    pub history: Vec<(u64, u32, u32, u32)>, // (timestamp, previous level, new level, adjustment signal)
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
        /// Baseline liquidity level for initialization.
        #[pallet::constant]
        type BaselineLiquidity: Get<u32>;
        /// Smoothing factor for liquidity adjustments.
        #[pallet::constant]
        type LiquiditySmoothingFactor: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn liquidity_state)]
    pub type LiquidityStateStorage<T: Config> = StorageValue<_, LiquidityState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when the liquidity level is updated.
        LiquidityUpdated(u32, u32, u32), // (previous level, new level, adjustment signal)
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The provided liquidity adjustment signal is invalid.
        InvalidAdjustmentSignal,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the liquidity state with a baseline level.
        pub fn initialize_liquidity() -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineLiquidity::get();
            let state = LiquidityState {
                current_level: baseline,
                history: vec![(now, 0, baseline, 0)],
            };
            <LiquidityStateStorage<T>>::put(state);
            Ok(())
        }

        /// Updates the liquidity level based on an adjustment signal.
        ///
        /// The new liquidity level is calculated as:
        ///    new_level = current_level + (adjustment_signal / smoothing_factor)
        pub fn update_liquidity(adjustment_signal: u32) -> DispatchResult {
            ensure!(adjustment_signal > 0, Error::<T>::InvalidAdjustmentSignal);
            let mut state = <LiquidityStateStorage<T>>::get();
            let previous = state.current_level;
            let adjustment = adjustment_signal / T::LiquiditySmoothingFactor::get();
            let new_level = previous.saturating_add(adjustment);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push((now, previous, new_level, adjustment_signal));
            state.current_level = new_level;
            <LiquidityStateStorage<T>>::put(state);
            Self::deposit_event(Event::LiquidityUpdated(previous, new_level, adjustment_signal));
            Ok(())
        }
    }
}
