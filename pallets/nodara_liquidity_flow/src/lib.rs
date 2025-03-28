#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Liquidity Flow Module - Advanced Version
//!
//! This module manages liquidity flows within the Nodara network. It monitors real-time liquidity,
//! adjusts parameters to redistribute funds dynamically, and maintains a full audit log of all liquidity
//! adjustments. DAO governance integration allows for future updates of base parameters. The module is
//! optimized for high-performance environments on testnet/mainnet.
//!
//! ## Advanced Features
//! - **Real-Time Liquidity Monitoring:** Tracks liquidity levels continuously.
//! - **Dynamic Adjustments:** Automatically adjusts liquidity based on measured metrics.
//! - **Audit Logging:** Records each liquidity adjustment with a timestamp, previous level, new level, and adjustment metric.
//! - **DAO Governance Integration:** Future-proof design for parameter updates via on-chain governance.
//! - **Performance Optimizations:** Optimized arithmetic operations and integrated benchmarks.
//!

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure representing a liquidity adjustment record.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct LiquidityRecord {
    pub timestamp: u64,
    pub previous_level: u32,
    pub new_level: u32,
    pub adjustment_metric: u32,
}

/// Global state for liquidity management.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct LiquidityState {
    pub current_level: u32,
    pub history: Vec<LiquidityRecord>,
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
        /// Niveau de liquidité de base pour l'initialisation.
        #[pallet::constant]
        type BaselineLiquidity: Get<u32>;
        /// Facteur de lissage pour les ajustements.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de liquidité.
    #[pallet::storage]
    #[pallet::getter(fn liquidity_state)]
    pub type LiquidityStateStorage<T: Config> = StorageValue<_, LiquidityState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors d'une mise à jour de liquidité.
        /// (niveau précédent, nouveau niveau, métrique d'ajustement)
        LiquidityUpdated(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La valeur de l'ajustement (métrique) est invalide.
        InvalidAdjustmentMetric,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de liquidité avec la valeur de base.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
            // Seule la racine (Root) est autorisée à initialiser l'état.
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineLiquidity::get();
            let state = LiquidityState {
                current_level: baseline,
                history: vec![LiquidityRecord {
                    timestamp,
                    previous_level: 0,
                    new_level: baseline,
                    adjustment_metric: 0,
                }],
            };
            <LiquidityStateStorage<T>>::put(state);
            Ok(())
        }

        /// Met à jour le niveau de liquidité en fonction d'une métrique d'ajustement.
        ///
        /// Le nouveau niveau est calculé ainsi :
        /// `new_level = current_level + (adjustment_metric / smoothing_factor)`
        #[pallet::weight(10_000)]
        pub fn update_liquidity(origin: OriginFor<T>, adjustment_metric: u32) -> DispatchResult {
            // Ici, nous acceptons un appel signé (peut être restreint par la suite).
            ensure_signed(origin)?;
            ensure!(adjustment_metric > 0, Error::<T>::InvalidAdjustmentMetric);

            let mut state = <LiquidityStateStorage<T>>::get();
            let previous_level = state.current_level;
            let adjustment = adjustment_metric / T::SmoothingFactor::get();
            let new_level = previous_level.saturating_add(adjustment);

            state.current_level = new_level;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(LiquidityRecord {
                timestamp,
                previous_level,
                new_level,
                adjustment_metric,
            });
            <LiquidityStateStorage<T>>::put(state);

            Self::deposit_event(Event::LiquidityUpdated(previous_level, new_level, adjustment_metric));
            Ok(())
        }
    }
}
