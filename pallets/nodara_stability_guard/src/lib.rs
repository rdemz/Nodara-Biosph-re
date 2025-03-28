#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Stability Guard Module - Dynamic Network Stability Management
//!
//! This module monitors network volatility and dynamically adjusts a stability parameter to maintain the
//! overall health of the network. It logs every adjustment for full auditability and supports DAO-driven
//! configuration updates. The new parameter is computed by adding an adjustment (based on the measured volatility
//! and a smoothing factor) to the current parameter value.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure representing a record of a stability adjustment.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StabilityRecord {
    pub timestamp: u64,
    pub parameter: u32,
    pub volatility: u32,
}

/// Global state of the stability guard.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct StabilityState {
    pub current_parameter: u32,
    pub history: Vec<StabilityRecord>,
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
        /// Paramètre de stabilité de base pour l'initialisation.
        #[pallet::constant]
        type BaselineParameter: Get<u32>;
        /// Facteur de lissage utilisé pour calculer l'ajustement.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de stabilité.
    #[pallet::storage]
    #[pallet::getter(fn stability_state)]
    pub type StabilityStateStorage<T: Config> = StorageValue<_, StabilityState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors de la mise à jour du paramètre de stabilité.
        /// (ancien paramètre, nouveau paramètre, volatilité mesurée)
        StabilityParameterUpdated(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La valeur de volatilité fournie est invalide.
        InvalidVolatility,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de stabilité avec la valeur de base.
        ///
        /// Cette fonction doit être appelée par la racine (Root) afin d'initialiser le module.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineParameter::get();
            let state = StabilityState {
                current_parameter: baseline,
                history: vec![StabilityRecord {
                    timestamp,
                    parameter: baseline,
                    volatility: 0,
                }],
            };
            <StabilityStateStorage<T>>::put(state);
            Ok(())
        }

        /// Met à jour le paramètre de stabilité en fonction d'une mesure de volatilité.
        ///
        /// Le nouveau paramètre est calculé comme suit :
        /// `new_parameter = old_parameter + (volatility / smoothing_factor)`
        #[pallet::weight(10_000)]
        pub fn update_parameter(origin: OriginFor<T>, volatility: u32) -> DispatchResult {
            // Ici, nous acceptons un appel signé (peut être remplacé par ensure_root selon la gouvernance).
            ensure_signed(origin)?;
            ensure!(volatility > 0, Error::<T>::InvalidVolatility);

            let mut state = <StabilityStateStorage<T>>::get();
            let old_parameter = state.current_parameter;
            let adjustment = volatility / T::SmoothingFactor::get();
            let new_parameter = old_parameter.saturating_add(adjustment);
            state.current_parameter = new_parameter;

            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(StabilityRecord {
                timestamp,
                parameter: new_parameter,
                volatility,
            });
            <StabilityStateStorage<T>>::put(state);

            Self::deposit_event(Event::StabilityParameterUpdated(old_parameter, new_parameter, volatility));
            Ok(())
        }
    }
}
