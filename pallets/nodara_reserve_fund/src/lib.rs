#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Reserve Fund Module - Advanced Version
//!
//! This module manages the reserve fund within the Nodara network. It collects contributions,
//! maintains a reserve balance, and redistributes funds when needed to stabilize the network economy.
//! It includes comprehensive audit logging and is designed to integrate with DAO governance for future parameter updates.
//!
//! ## Advanced Features:
//! - **Fund Collection & Redistribution:** Securely collects funds (e.g. transaction fees) and allows controlled withdrawals.
//! - **Dynamic Reserve Management:** Adjusts the reserve balance in real time based on contributions and withdrawals.
//! - **Audit Logging:** Maintains an immutable log of every operation with timestamps and details.
//! - **DAO Governance Integration:** Permits future mises à jour des paramètres via des propositions on-chain.
//! - **Performance Optimizations:** Utilisation d'opérations arithmétiques optimisées pour un traitement rapide.
//!

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure d'un enregistrement d'opération sur le fonds de réserve.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ReserveRecord {
    pub timestamp: u64,
    pub previous_balance: u128,
    pub new_balance: u128,
    pub operation: Vec<u8>, // Description ou raison de l'opération.
}

/// État global du fonds de réserve.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct ReserveFundState {
    pub balance: u128,
    pub history: Vec<ReserveRecord>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Solde de réserve de base pour l'initialisation.
        #[pallet::constant]
        type BaselineReserve: Get<u128>;
    }

    /// Stockage de l'état du fonds de réserve.
    #[pallet::storage]
    #[pallet::getter(fn reserve_state)]
    pub type ReserveFundStorage<T: Config> = StorageValue<_, ReserveFundState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors d'une mise à jour du fonds de réserve.
        /// (solde précédent, nouveau solde, description de l'opération)
        ReserveUpdated(u128, u128, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Opération invalide (par exemple, tentative de retrait supérieure au solde disponible).
        InvalidOperation,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise le fonds de réserve avec le solde de base.
        /// Seul un appel provenant de l'origine `Root` est autorisé.
        #[pallet::weight(10_000)]
        pub fn initialize_reserve(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineReserve::get();
            let state = ReserveFundState {
                balance: baseline,
                history: vec![ReserveRecord {
                    timestamp,
                    previous_balance: 0,
                    new_balance: baseline,
                    operation: b"Initialization".to_vec(),
                }],
            };
            <ReserveFundStorage<T>>::put(state);
            Ok(())
        }

        /// Ajoute une contribution au fonds de réserve.
        ///
        /// La contribution est ajoutée au solde actuel et enregistrée dans l'historique.
        #[pallet::weight(10_000)]
        pub fn contribute(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            // Un appel signé est requis.
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_add(amount);
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(ReserveRecord {
                timestamp,
                previous_balance,
                new_balance: state.balance,
                operation: description.clone(),
            });
            <ReserveFundStorage<T>>::put(state);
            Self::deposit_event(Event::ReserveUpdated(previous_balance, state.balance, description));
            Ok(())
        }

        /// Effectue un retrait du fonds de réserve.
        ///
        /// Le retrait est autorisé uniquement si le solde est suffisant.
        #[pallet::weight(10_000)]
        pub fn withdraw(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            // Un appel signé est requis.
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            ensure!(state.balance >= amount, Error::<T>::InvalidOperation);
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_sub(amount);
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(ReserveRecord {
                timestamp,
                previous_balance,
                new_balance: state.balance,
                operation: description.clone(),
            });
            <ReserveFundStorage<T>>::put(state);
            Self::deposit_event(Event::ReserveUpdated(previous_balance, state.balance, description));
            Ok(())
        }
    }
}
