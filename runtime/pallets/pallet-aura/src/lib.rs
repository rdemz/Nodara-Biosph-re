// runtime/pallets/pallet-aura/src/lib.rs - Pallet Aura (Legendary Edition)

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # pallet_aura
//!
//! Ce pallet implémente le mécanisme de production de blocs Aura pour le runtime.
//! Il gère la validation des blocs par les autorités et expose des fonctions (ex. pour définir
//! la clé d'autorité, principalement pour les tests ou la configuration initiale).
//!
//! En production, ce pallet est utilisé pour autoriser la création de blocs par les autorités définies.

use frame_support::{pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
extern crate serde;

/// Pallet Aura
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du pallet Aura.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Clé d'autorité utilisée pour l'Aura.
        type AuthorityId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord;
        /// Poids pour les extrinsics (placeholder).
        type WeightInfo: WeightInfo;
    }

    /// Trait de poids pour le pallet Aura.
    pub trait WeightInfo {
        fn set_authority() -> Weight;
    }

    #[cfg(feature = "std")]
    impl WeightInfo for () {
        fn set_authority() -> Weight {
            0
        }
    }

    /// Storage pour la clé d'autorité actuelle (optionnel, pour tests ou configuration).
    #[pallet::storage]
    #[pallet::getter(fn authority)]
    pub type Authority<T: Config> = StorageValue<_, T::AuthorityId, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Extrinsic pour définir la clé d'autorité Aura.
        /// Seul Root peut l'appeler. Cette extrinsic est surtout destinée aux tests ou à la configuration initiale.
        #[pallet::weight(T::WeightInfo::set_authority())]
        pub fn set_authority(origin: OriginFor<T>, authority: T::AuthorityId) -> DispatchResult {
            ensure_root(origin)?;
            <Authority<T>>::put(authority.clone());
            Self::deposit_event(Event::AuthoritySet(authority));
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Clé d'autorité définie pour Aura.
        AuthoritySet(T::AuthorityId),
    }
}
