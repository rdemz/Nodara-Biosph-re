#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Liquidity Flow Module - Advanced Version
//!
//! Ce module gère les flux de liquidité au sein du réseau Nodara. Il surveille en temps réel le niveau de liquidité,
//! ajuste dynamiquement les paramètres pour redistribuer les fonds, et conserve un journal complet de toutes les opérations
//! d'ajustement. Ce module intègre des optimisations pour les environnements à haute performance (testnet/mainnet).

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::{Currency, Get},
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Structure représentant un enregistrement d'ajustement de liquidité.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct LiquidityRecord {
    /// Horodatage de l'opération.
    pub timestamp: u64,
    /// Niveau de liquidité avant l'ajustement.
    pub previous_level: u32,
    /// Niveau de liquidité après l'ajustement.
    pub new_level: u32,
    /// Valeur de la métrique d'ajustement fournie.
    pub adjustment_metric: u32,
}

/// État global du module de liquidité.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct LiquidityState {
    /// Niveau de liquidité actuel.
    pub current_level: u32,
    /// Historique complet des ajustements.
    pub history: Vec<LiquidityRecord>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::Zero;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Niveau de liquidité de base pour l'initialisation.
        #[pallet::constant]
        type BaselineLiquidity: Get<u32>;
        /// Facteur de lissage pour le calcul de l'ajustement.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état de liquidité.
    #[pallet::storage]
    #[pallet::getter(fn liquidity_state)]
    pub type LiquidityStateStorage<T: Config> = StorageValue<_, LiquidityState, ValueQuery>;

    /// Configuration de genèse pour pré‑initialiser l'état de liquidité.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub liquidity_state: Option<LiquidityState>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { liquidity_state: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if let Some(state) = &self.liquidity_state {
                <LiquidityStateStorage<T>>::put(state.clone());
            } else {
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
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors d'une mise à jour de liquidité.
        /// (niveau précédent, nouveau niveau, métrique d'ajustement)
        LiquidityUpdated(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La valeur de l'ajustement (métrique) doit être supérieure à zéro.
        InvalidAdjustmentMetric,
        /// Le facteur de lissage ne peut pas être nul.
        ZeroSmoothingFactor,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de liquidité avec la valeur de base.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
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
        /// Le nouveau niveau est calculé par :
        ///     new_level = current_level + (adjustment_metric / smoothing_factor)
        #[pallet::weight(10_000)]
        pub fn update_liquidity(origin: OriginFor<T>, adjustment_metric: u32) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(adjustment_metric > 0, Error::<T>::InvalidAdjustmentMetric);

            let smoothing = T::SmoothingFactor::get();
            ensure!(smoothing != 0, Error::<T>::ZeroSmoothingFactor);

            let mut state = <LiquidityStateStorage<T>>::get();
            let previous_level = state.current_level;
            let adjustment = adjustment_metric / smoothing;
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

    impl<T: Config> Pallet<T> {
        /// Retourne un horodatage fixe.
        /// En production, remplacez par `pallet_timestamp` pour obtenir un temps réel.
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, assert_err, parameter_types};
        use sp_core::H256;
        use sp_runtime::{
            traits::{BlakeTwo256, IdentityLookup},
            testing::Header,
        };
        use frame_system as system;

        type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
        type Block = system::mocking::MockBlock<Test>;

        frame_support::construct_runtime!(
            pub enum Test where 
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                LiquidityFlowModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineLiquidity: u32 = 1000;
            pub const SmoothingFactor: u32 = 10;
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
            type BaselineLiquidity = BaselineLiquidity;
            type SmoothingFactor = SmoothingFactor;
        }

        #[test]
        fn test_initialize_state() {
            let origin = system::RawOrigin::Root.into();
            assert_ok!(LiquidityFlowModule::initialize_state(origin));
            let state = LiquidityFlowModule::liquidity_state();
            assert_eq!(state.current_level, BaselineLiquidity::get());
            assert_eq!(state.history.len(), 1);
            let record = &state.history[0];
            assert_eq!(record.new_level, BaselineLiquidity::get());
        }

        #[test]
        fn test_update_liquidity() {
            let root_origin = system::RawOrigin::Root.into();
            assert_ok!(LiquidityFlowModule::initialize_state(root_origin));
            let initial_state = LiquidityFlowModule::liquidity_state();
            let initial_level = initial_state.current_level;
            // Avec adjustment_metric = 50 et SmoothingFactor = 10, l'ajustement sera 50 / 10 = 5.
            let adjustment_metric = 50;
            assert_ok!(LiquidityFlowModule::update_liquidity(system::RawOrigin::Signed(1).into(), adjustment_metric));
            let new_state = LiquidityFlowModule::liquidity_state();
            assert_eq!(new_state.current_level, initial_level + 5);
            assert_eq!(new_state.history.len(), 2);
        }

        #[test]
        fn test_update_liquidity_fail_invalid_adjustment() {
            assert_err!(
                LiquidityFlowModule::update_liquidity(system::RawOrigin::Signed(1).into(), 0),
                Error::<Test>::InvalidAdjustmentMetric
            );
        }
    }
}

