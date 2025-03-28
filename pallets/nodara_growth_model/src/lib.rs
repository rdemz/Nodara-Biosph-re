#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Growth Model Module - Dynamic Growth Incentives
//!
//! This module implements dynamic growth incentives for the Nodara network by adjusting a reward multiplier
//! based on a network signal. It logs chaque mise à jour pour garantir la traçabilité et intègre des paramètres
//! modulables via DAO governance.
//!
//! The module exposes functions to initialize the state and update the growth multiplier based on a provided signal.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure regroupant les données de croissance.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct GrowthData {
    pub multiplier: u32,
    pub signal: u32,
    pub timestamp: u64,
}

/// État global du module de croissance.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct GrowthState {
    pub current_multiplier: u32,
    pub history: Vec<GrowthData>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Multiplicateur de base pour l'initialisation.
        #[pallet::constant]
        type BaselineMultiplier: Get<u32>;
        /// Facteur de lissage pour éviter des ajustements trop brusques.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de croissance.
    #[pallet::storage]
    #[pallet::getter(fn growth_state)]
    pub type GrowthStateStorage<T: Config> = StorageValue<_, GrowthState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Émission lors de la mise à jour du multiplicateur de croissance (ancien, nouveau, signal).
        GrowthMultiplierUpdated(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Signal invalide.
        InvalidSignal,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de croissance avec la valeur de base.
        ///
        /// Cette fonction doit être appelée par la racine (Root) pour initialiser le module.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineMultiplier::get();
            let state = GrowthState {
                current_multiplier: baseline,
                history: vec![GrowthData {
                    multiplier: baseline,
                    signal: 0,
                    timestamp,
                }],
            };
            <GrowthStateStorage<T>>::put(state);
            Ok(())
        }

        /// Met à jour le multiplicateur de croissance en fonction du signal fourni.
        ///
        /// Le nouveau multiplicateur est calculé comme suit :
        /// `new_multiplier = old_multiplier + (signal / smoothing_factor)`
        #[pallet::weight(10_000)]
        pub fn update_multiplier(origin: OriginFor<T>, signal: u32) -> DispatchResult {
            // Ici, nous acceptons un appel signé pour permettre à un utilisateur autorisé de déclencher l'update.
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidSignal);

            let mut state = <GrowthStateStorage<T>>::get();
            let old_multiplier = state.current_multiplier;
            // Calcul simple de l'ajustement.
            let adjustment = signal / T::SmoothingFactor::get();
            let new_multiplier = old_multiplier.saturating_add(adjustment);
            state.current_multiplier = new_multiplier;

            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(GrowthData {
                multiplier: new_multiplier,
                signal,
                timestamp,
            });
            <GrowthStateStorage<T>>::put(state);

            Self::deposit_event(Event::GrowthMultiplierUpdated(old_multiplier, new_multiplier, signal));
            Ok(())
        }
    }
}
