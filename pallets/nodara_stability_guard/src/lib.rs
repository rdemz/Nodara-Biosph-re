#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Stability Guard Module - Extreme Version
//!
//! Ce module surveille la volatilité du réseau et ajuste dynamiquement un paramètre de stabilité pour préserver la santé globale du réseau.
//! Il utilise une moyenne mobile exponentielle (EMA) pour lisser les mesures de volatilité et détermine l'ajustement à appliquer en fonction
//! de la variation de cette EMA. Le nouveau paramètre est ensuite contraint par des bornes minimales et maximales.
//! Chaque ajustement est enregistré dans un historique pour assurer une traçabilité complète, et le module est conçu pour intégrer
//! des mises à jour de configuration via DAO.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure représentant un enregistrement d'ajustement de stabilité.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StabilityRecord {
    pub timestamp: u64,
    pub old_parameter: u32,
    pub new_parameter: u32,
    pub volatility: u32,
    pub new_ema: u32,
}

/// État global du module de stabilité.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct StabilityState {
    pub current_parameter: u32,
    /// Moyenne mobile exponentielle de la volatilité.
    pub volatility_ema: u32,
    pub history: Vec<StabilityRecord>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::SaturatedConversion;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Paramètre de stabilité initial (valeur de base).
        #[pallet::constant]
        type BaselineParameter: Get<u32>;
        /// Facteur de lissage utilisé pour le calcul de l'EMA (doit être > 0).
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
        /// Valeur maximale autorisée pour le paramètre de stabilité.
        #[pallet::constant]
        type MaxStabilityParameter: Get<u32>;
        /// Valeur minimale autorisée pour le paramètre de stabilité.
        #[pallet::constant]
        type MinStabilityParameter: Get<u32>;
    }

    /// Stockage de l'état de stabilité.
    #[pallet::storage]
    #[pallet::getter(fn stability_state)]
    pub type StabilityStateStorage<T: Config> = StorageValue<_, StabilityState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors de la mise à jour du paramètre de stabilité.
        /// (ancien paramètre, nouveau paramètre, volatilité mesurée, nouvelle EMA)
        StabilityParameterUpdated(u32, u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La valeur de volatilité fournie est invalide.
        InvalidVolatility,
        /// Le facteur de lissage doit être supérieur à zéro.
        ZeroSmoothingFactor,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de stabilité avec le paramètre de base et une EMA initiale nulle.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineParameter::get();
            let state = StabilityState {
                current_parameter: baseline,
                volatility_ema: 0,
                history: vec![StabilityRecord {
                    timestamp,
                    old_parameter: baseline,
                    new_parameter: baseline,
                    volatility: 0,
                    new_ema: 0,
                }],
            };
            <StabilityStateStorage<T>>::put(state);
            Ok(())
        }

        /// Met à jour le paramètre de stabilité en fonction d'une mesure de volatilité.
        ///
        /// La mise à jour s'effectue en deux étapes :
        /// 1. Calcul de la nouvelle EMA via :  
        ///    new_ema = (volatility + (smoothing_factor - 1) * old_ema) / smoothing_factor  
        /// 2. L'ajustement appliqué au paramètre est la différence entre new_ema et old_ema.  
        ///    new_parameter = old_parameter + (new_ema - old_ema)
        /// Enfin, le paramètre est limité par les bornes min et max.
        #[pallet::weight(10_000)]
        pub fn update_parameter(origin: OriginFor<T>, volatility: u32) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(volatility > 0, Error::<T>::InvalidVolatility);
            let smoothing = T::SmoothingFactor::get();
            ensure!(smoothing > 0, Error::<T>::ZeroSmoothingFactor);

            let mut state = <StabilityStateStorage<T>>::get();
            let old_parameter = state.current_parameter;
            let old_ema = state.volatility_ema;
            // Calcul de la nouvelle EMA.
            let new_ema = if old_ema == 0 {
                // Si aucune EMA précédente, on initialise avec la volatilité actuelle.
                volatility
            } else {
                (volatility + (smoothing - 1) * old_ema) / smoothing
            };
            state.volatility_ema = new_ema;
            // L'ajustement est la différence entre la nouvelle et l'ancienne EMA.
            let adjustment = new_ema.saturating_sub(old_ema);
            let mut new_parameter = old_parameter.saturating_add(adjustment);
            // Limiter le paramètre aux bornes définies.
            if new_parameter > T::MaxStabilityParameter::get() {
                new_parameter = T::MaxStabilityParameter::get();
            }
            if new_parameter < T::MinStabilityParameter::get() {
                new_parameter = T::MinStabilityParameter::get();
            }
            state.current_parameter = new_parameter;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(StabilityRecord {
                timestamp,
                old_parameter,
                new_parameter,
                volatility,
                new_ema,
            });
            <StabilityStateStorage<T>>::put(state);

            Self::deposit_event(Event::StabilityParameterUpdated(old_parameter, new_parameter, volatility, new_ema));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Retourne un horodatage fixe.
        /// En production, remplacez cette fonction par un fournisseur de temps fiable (ex. `pallet_timestamp`).
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}
