#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_reserve_fund - Legendary Edition
//!
//! This module manages the reserve fund for Nodara BIOSPHÈRE QUANTIC. It collects a designated portion of transaction fees
//! and other revenues, ensuring that the network maintains a stable financial buffer. The module includes mechanisms for controlled
//! fund distribution, invariant checks, and full audit logging for transparency and security.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing the reserve fund state.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ReserveState {
    pub balance: u32,
    pub history: Vec<(u64, u32, u32, Vec<u8>)>, // (timestamp, previous balance, new balance, operation reason/signal)
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
        /// Baseline reserve balance.
        #[pallet::constant]
        type BaselineReserve: Get<u32>;
        /// Minimum reserve level required.
        #[pallet::constant]
        type MinReserve: Get<u32>;
        /// Maximum reserve level allowed.
        #[pallet::constant]
        type MaxReserve: Get<u32>;
        /// Smoothing factor for automatic reserve adjustments.
        #[pallet::constant]
        type ReserveSmoothingFactor: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn reserve_state)]
    pub type ReserveStateStorage<T: Config> = StorageValue<_, ReserveState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when the reserve balance is updated.
        ReserveUpdated(u32, u32, Vec<u8>), // (previous balance, new balance, operation reason/signal)
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The withdrawal amount exceeds the current reserve balance.
        InsufficientReserve,
        /// The provided adjustment signal is invalid.
        InvalidAdjustmentSignal,
    }

    impl<T: Config> Pallet<T> {
        /// Initializes the reserve state with the baseline reserve balance.
        pub fn initialize_reserve() -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineReserve::get();
            let state = ReserveState {
                balance: baseline,
                history: vec![(now, 0, baseline, b"Initialization".to_vec())],
            };
            <ReserveStateStorage<T>>::put(state);
            Ok(())
        }

        /// Contributes funds to the reserve. This function simulates the collection of fees.
        pub fn contribute(amount: u32, reason: Vec<u8>) -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            <ReserveStateStorage<T>>::mutate(|state| {
                let previous = state.balance;
                let new_balance = previous.saturating_add(amount);
                state.history.push((now, previous, new_balance, reason.clone()));
                state.balance = new_balance;
            });
            Self::deposit_event(Event::ReserveUpdated(
                <ReserveStateStorage<T>>::get().history.last().unwrap().1,
                <ReserveStateStorage<T>>::get().balance,
                reason,
            ));
            Ok(())
        }

        /// Withdraws funds from the reserve, ensuring the balance does not fall below the minimum reserve.
        pub fn withdraw(amount: u32, reason: Vec<u8>) -> DispatchResult {
            <ReserveStateStorage<T>>::try_mutate(|state| -> DispatchResult {
                let previous = state.balance;
                ensure!(previous >= amount, Error::<T>::InsufficientReserve);
                let new_balance = previous.saturating_sub(amount);
                ensure!(new_balance >= T::MinReserve::get(), Error::<T>::InsufficientReserve);
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                state.history.push((now, previous, new_balance, reason.clone()));
                state.balance = new_balance;
                Ok(())
            })?;
            Self::deposit_event(Event::ReserveUpdated(
                <ReserveStateStorage<T>>::get().history.last().unwrap().1,
                <ReserveStateStorage<T>>::get().balance,
                reason,
            ));
            Ok(())
        }

        /// Updates the reserve balance automatically based on an economic adjustment signal.
        ///
        /// The new balance is computed using a smoothing algorithm:
        ///   new_balance = current_balance + (signal / ReserveSmoothingFactor)
        pub fn update_reserve(signal: u32, reason: Vec<u8>) -> DispatchResult {
            ensure!(signal > 0, Error::<T>::InvalidAdjustmentSignal);
            <ReserveStateStorage<T>>::mutate(|state| {
                let previous = state.balance;
                let adjustment = signal / T::ReserveSmoothingFactor::get();
                let new_balance = previous.saturating_add(adjustment);
                ensure!(new_balance <= T::MaxReserve::get(), Error::<T>::InvalidAdjustmentSignal);
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                state.history.push((now, previous, new_balance, reason.clone()));
                state.balance = new_balance;
            });
            Self::deposit_event(Event::ReserveUpdated(
                <ReserveStateStorage<T>>::get().history.last().unwrap().1,
                <ReserveStateStorage<T>>::get().balance,
                reason,
            ));
            Ok(())
        }
    }
}
