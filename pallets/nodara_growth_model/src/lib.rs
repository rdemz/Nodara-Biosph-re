#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Growth Model Module - Dynamic Growth Incentives (Extreme Edition)
//!
//! Ce module gère l'état adaptatif du réseau Nodara en ajustant dynamiquement le multiplicateur de croissance
//! en fonction d'un signal. Il intègre des vérifications renforcées, une gestion complète de l'historique des mises à jour
//! et une configuration de genèse pour pré-enregistrer des actifs ou des paramètres par défaut.
//!
//! Le nouveau multiplicateur est calculé simplement par :
//!     new_multiplier = old_multiplier + (signal / smoothing_factor)
//!
//! Des vérifications garantissent que le signal et le facteur de lissage sont valides.

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::Get,
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;
use sp_std::prelude::*; // Pour inclure ToString et autres traits utiles
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
    use sp_runtime::traits::Zero;

    /// Type représentant l'identifiant d'un actif (ex : b"BTC", b"ETH", etc.).
    pub type AssetId = Vec<u8>;
    /// Type représentant l'identifiant d'un transfert.
    pub type TransferId = u64;

    /// Métadonnées d'un actif supporté par le bridge.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct AssetMetadata {
        pub name: Vec<u8>,
        pub symbol: Vec<u8>,
        pub decimals: u8,
        pub source_chain: Vec<u8>,
    }

    /// Structure représentant une demande de transfert inter‑chaînes.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct TransferRequest<AccountId> {
        pub id: TransferId,
        pub from: AccountId,
        pub asset: AssetId,
        pub amount: u128,
        pub destination: AccountId,
        pub confirmations: BTreeSet<AccountId>,
        pub to_nodara: bool,
    }

    /// État global de la biosphère.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct BioState {
        pub current_multiplier: u32,
        pub history: Vec<GrowthData>,
    }

    /// Enumération des phases opérationnelles (exemple pour une extension future).
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum BioPhase {
        Growth,
        Defense,
        Mutation,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Multiplicateur de base pour l'initialisation.
        #[pallet::constant]
        type BaselineMultiplier: Get<u32>;
        /// Facteur de lissage pour éviter des ajustements trop brusques (ne doit pas être zéro).
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
        /// Nombre minimum de confirmations requis pour finaliser un transfert.
        #[pallet::constant]
        type RequiredConfirmations: Get<u32>;
        /// Gestionnaire des tokens représentatifs pour le bridge.
        type AssetManager: super::BridgeAssetManager<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Stockage des actifs supportés par le bridge.
    #[pallet::storage]
    #[pallet::getter(fn supported_assets)]
    pub type SupportedAssets<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetId, AssetMetadata, OptionQuery>;

    /// Stockage des demandes de transfert en attente.
    #[pallet::storage]
    #[pallet::getter(fn pending_transfers)]
    pub type PendingTransfers<T: Config> =
        StorageMap<_, Blake2_128Concat, TransferId, TransferRequest<T::AccountId>, OptionQuery>;

    /// Compteur pour générer des identifiants uniques de transfert.
    #[pallet::storage]
    #[pallet::getter(fn next_transfer_id)]
    pub type NextTransferId<T: Config> = StorageValue<_, TransferId, ValueQuery>;

    /// Stockage de l'état global de la biosphère.
    #[pallet::storage]
    #[pallet::getter(fn bio_state)]
    pub type BioStateStorage<T: Config> = StorageValue<_, BioState, ValueQuery>;

    /// Configuration de genèse permettant de pré-enregistrer des actifs supportés.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_assets: Vec<(AssetId, AssetMetadata)>,
        pub initial_growth_state: Option<GrowthState>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_assets: vec![
                    (b"BTC".to_vec(), AssetMetadata { name: b"Bitcoin".to_vec(), symbol: b"BTC".to_vec(), decimals: 8, source_chain: b"BTC".to_vec() }),
                    (b"ETH".to_vec(), AssetMetadata { name: b"Ethereum".to_vec(), symbol: b"ETH".to_vec(), decimals: 18, source_chain: b"ETH".to_vec() }),
                    // ... autres actifs ...
                ],
                initial_growth_state: None,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (asset_id, metadata) in &self.initial_assets {
                SupportedAssets::<T>::insert(asset_id, metadata);
            }
            if let Some(state) = &self.initial_growth_state {
                <BioStateStorage<T>>::put(state.clone());
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
                <BioStateStorage<T>>::put(state);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Un actif a été enregistré dans le bridge. [asset_id]
        AssetRegistered(AssetId),
        /// Une demande de transfert a été initiée. [transfer_id, from, asset, amount, destination, direction]
        TransferInitiated(TransferId, T::AccountId, AssetId, u128, T::AccountId, bool),
        /// Un validateur a confirmé un transfert. [transfer_id, validateur]
        TransferConfirmed(TransferId, T::AccountId),
        /// Un transfert a été finalisé et exécuté (mint ou burn). [transfer_id]
        TransferFinalized(TransferId),
        /// Mise à jour de l'état de croissance (ancien multiplicateur, nouveau multiplicateur, signal)
        GrowthMultiplierUpdated(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// L’actif n’est pas supporté par le bridge.
        AssetNotSupported,
        /// La demande de transfert est introuvable.
        TransferNotFound,
        /// Le nombre de confirmations est insuffisant pour finaliser le transfert.
        InsufficientConfirmations,
        /// Le validateur a déjà confirmé ce transfert.
        AlreadyConfirmed,
        /// L’ID d’actif ou les métadonnées sont invalides.
        InvalidAssetDefinition,
        /// Le montant doit être supérieur à zéro.
        InvalidAmount,
        /// Signal invalide.
        InvalidSignal,
        /// Facteur de lissage ne peut pas être zéro.
        ZeroSmoothingFactor,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de croissance avec la valeur de base.
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
            <BioStateStorage<T>>::put(state);
            Ok(())
        }

        /// Met à jour le multiplicateur de croissance en fonction du signal fourni.
        ///
        /// Le nouveau multiplicateur est calculé comme suit :
        /// `new_multiplier = old_multiplier + (signal / smoothing_factor)`
        #[pallet::weight(10_000)]
        pub fn update_multiplier(origin: OriginFor<T>, signal: u32) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidSignal);

            let smoothing = T::SmoothingFactor::get();
            ensure!(smoothing != 0, Error::<T>::ZeroSmoothingFactor);

            let mut state = <BioStateStorage<T>>::get();
            let old_multiplier = state.current_multiplier;
            let adjustment = signal / smoothing;
            let new_multiplier = old_multiplier.saturating_add(adjustment);
            state.current_multiplier = new_multiplier;

            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(GrowthData {
                multiplier: new_multiplier,
                signal,
                timestamp,
            });
            <BioStateStorage<T>>::put(state);

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
                Biosphere: Pallet,
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
            // Pour la genèse, nous utilisons la valeur de base par défaut définie dans la configuration.
        }

        #[test]
        fn test_initialize_state() {
            let origin = system::RawOrigin::Root.into();
            assert_ok!(Biosphere::initialize_state(origin));
            let state = Biosphere::growth_state();
            assert_eq!(state.current_multiplier, BaselineMultiplier::get());
            assert_eq!(state.history.len(), 1);
        }

        #[test]
        fn test_update_multiplier() {
            let root_origin = system::RawOrigin::Root.into();
            assert_ok!(Biosphere::initialize_state(root_origin));
            let signed_origin = system::RawOrigin::Signed(1).into();
            // Avec signal = 50 et facteur de lissage = 5, l'ajustement sera 50 / 5 = 10.
            assert_ok!(Biosphere::update_multiplier(signed_origin, 50));
            let state = Biosphere::growth_state();
            assert_eq!(state.current_multiplier, BaselineMultiplier::get() + 10);
            assert_eq!(state.history.len(), 2);
        }
    }
}
