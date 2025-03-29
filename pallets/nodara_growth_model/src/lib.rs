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

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Multiplicateur de base pour l'initialisation.
        #[pallet::constant]
        type BaselineMultiplier: Get<u32>;
        /// Facteur de lissage pour éviter des ajustements trop brusques (ne doit pas être zéro).
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de croissance.
    #[pallet::storage]
    #[pallet::getter(fn growth_state)]
    pub type GrowthStateStorage<T: Config> = StorageValue<_, GrowthState, ValueQuery>;

    /// Configuration de genèse pour pré-initialiser l'état.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub growth_state: Option<GrowthState>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { growth_state: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if let Some(state) = &self.growth_state {
                <GrowthStateStorage<T>>::put(state.clone());
            } else {
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
            }
        }
    }

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
        /// Facteur de lissage ne peut pas être zéro.
        ZeroSmoothingFactor,
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
            // Seul un appel signé est autorisé.
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidSignal);

            // Vérification du facteur de lissage.
            let smoothing = T::SmoothingFactor::get();
            ensure!(smoothing != 0, Error::<T>::ZeroSmoothingFactor);

            let mut state = <GrowthStateStorage<T>>::get();
            let old_multiplier = state.current_multiplier;
            // Calcul de l'ajustement.
            let adjustment = signal / smoothing;
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, parameter_types};
        use sp_core::H256;
        use sp_runtime::{
            traits::{BlakeTwo256, IdentityLookup},
            testing::Header,
        };
        use frame_system as system;

        type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
        type Block = frame_system::mocking::MockBlock<Test>;

        frame_support::construct_runtime!(
            pub enum Test where 
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                GrowthModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineMultiplier: u32 = 100;
            pub const SmoothingFactor: u32 = 5;
        }

        impl system::Config for Test {
            type BaseCallFilter = frame_support::traits::Everything;
            type BlockWeights = ();
            type BlockLength = ();
            type DbWeight = ();
            type RuntimeOrigin = system::mocking::Origin;
            type RuntimeCall = Call;
            type Index = u64;
            type BlockNumber = u64;
            type Hash = H256;
            type Hashing = BlakeTwo256;
            type AccountId = u64;
            type Lookup = IdentityLookup<Self::AccountId>;
            type Header = Header;
            type RuntimeEvent = ();
            type BlockHashCount = BlockHashCount;
            type Version = ();
            type PalletInfo = ();
            type AccountData = ();
            type OnNewAccount = ();
            type OnKilledAccount = ();
            type SystemWeightInfo = ();
            type SS58Prefix = ();
            type OnSetCode = ();
            type MaxConsumers = ();
        }

        impl Config for Test {
            type RuntimeEvent = ();
            type BaselineMultiplier = BaselineMultiplier;
            type SmoothingFactor = SmoothingFactor;
        }

        #[test]
        fn test_initialize_state() {
            // Appel depuis Root.
            let origin = system::RawOrigin::Root.into();
            assert_ok!(GrowthModule::initialize_state(origin));

            // Vérification que l'état a été initialisé avec la valeur de base.
            let state = GrowthModule::growth_state();
            assert_eq!(state.current_multiplier, BaselineMultiplier::get());
            assert_eq!(state.history.len(), 1);
        }

        #[test]
        fn test_update_multiplier() {
            // Initialisation de l'état.
            let root_origin = system::RawOrigin::Root.into();
            assert_ok!(GrowthModule::initialize_state(root_origin));

            // Mise à jour avec un signal valide.
            let signed_origin = system::RawOrigin::Signed(1).into();
            // Avec signal = 50 et facteur de lissage = 5, l'ajustement sera 50 / 5 = 10.
            assert_ok!(GrowthModule::update_multiplier(signed_origin, 50));

            // Vérification que l'état a été mis à jour.
            let state = GrowthModule::growth_state();
            assert_eq!(state.current_multiplier, BaselineMultiplier::get() + 10);
            assert_eq!(state.history.len(), 2);
        }
    }
}
