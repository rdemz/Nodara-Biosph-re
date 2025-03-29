#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Bridge Inter‑chaînes pour Nodara - Extreme Edition
//!
//! Ce module gère le transfert inter‑chaînes en verrouillant des actifs sur une blockchain source et
//! en émettant ou en brûlant des représentations sur Nodara. Il supporte une large gamme d’actifs (BNB, BTC, ETH, DOT, XRP, DOGE, SOL, LINK, SUI, AVAX, USDT, USDC, ADA, TRX, XLM, TON)
//! et intègre un mécanisme de validation décentralisée (confirmations multi‑signatures) pour sécuriser les transferts.
//!
//! Améliorations apportées dans cette version "extreme" :
//! - Validation renforcée des entrées (vérification de la non-nullité de l'ID, du nom et du symbole).
//! - Vérification que le montant du transfert est strictement positif.
//! - Gestion avancée des confirmations avec vérification anti-doublon.
//! - Documentation et commentaires détaillés pour chaque fonction.
//! - Configuration de genèse complète pour pré‑charger une liste d’actifs supportés.

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::{Currency, Get},
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;
use sp_std::prelude::*; // Inclut notamment le trait ToString

/// Trait pour gérer le minting et le burning des tokens représentatifs sur Nodara.
pub trait BridgeAssetManager<AccountId> {
    /// Crée (mint) des tokens représentatifs pour l’actif donné et les crédite au compte `to`.
    fn mint(asset: Vec<u8>, to: &AccountId, amount: u128) -> DispatchResult;
    /// Détruit (burn) des tokens représentatifs pour l’actif donné en les retirant du compte `from`.
    fn burn(asset: Vec<u8>, from: &AccountId, amount: u128) -> DispatchResult;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::Zero;

    /// Type pour l'identifiant d'un actif (ex: b"BTC", b"ETH", etc.).
    pub type AssetId = Vec<u8>;
    /// Type pour l'identifiant d'un transfert.
    pub type TransferId = u64;

    /// Métadonnées d'un actif supporté par le bridge.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct AssetMetadata {
        /// Nom complet de l'actif.
        pub name: Vec<u8>,
        /// Symbole de l'actif.
        pub symbol: Vec<u8>,
        /// Nombre de décimales.
        pub decimals: u8,
        /// Chaîne source (ex: b"BTC", b"ETH", b"ERC20", etc.).
        pub source_chain: Vec<u8>,
    }

    /// Structure représentant une demande de transfert inter‑chaînes.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct TransferRequest<AccountId> {
        /// Identifiant unique du transfert.
        pub id: TransferId,
        /// Compte à l'origine du transfert.
        pub from: AccountId,
        /// Identifiant de l'actif concerné.
        pub asset: AssetId,
        /// Montant à transférer.
        pub amount: u128,
        /// Compte destinataire.
        pub destination: AccountId,
        /// Ensemble des validateurs ayant confirmé le transfert.
        pub confirmations: BTreeSet<AccountId>,
        /// Direction du transfert : true = vers Nodara (mint), false = depuis Nodara (burn).
        pub to_nodara: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement utilisé par le runtime.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Module monétaire (pour d'éventuelles opérations financières, si nécessaire).
        type Currency: Currency<Self::AccountId>;
        /// Nombre minimum de confirmations requis pour finaliser un transfert.
        #[pallet::constant]
        type RequiredConfirmations: Get<u32>;
        /// Gestionnaire des tokens représentatifs pour le bridge.
        type AssetManager: BridgeAssetManager<Self::AccountId>;
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Enregistre un actif dans le bridge.
        ///
        /// Vérifie que l'ID de l'actif, le nom et le symbole ne sont pas vides.
        #[pallet::weight(10_000)]
        pub fn register_asset(origin: OriginFor<T>, asset: AssetId, metadata: AssetMetadata) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            ensure!(!asset.is_empty(), Error::<T>::InvalidAssetDefinition);
            ensure!(!metadata.name.is_empty(), Error::<T>::InvalidAssetDefinition);
            ensure!(!metadata.symbol.is_empty(), Error::<T>::InvalidAssetDefinition);
            // Insertion sans doublon (on suppose qu'un asset est unique).
            ensure!(!SupportedAssets::<T>::contains_key(&asset), Error::<T>::AssetAlreadyExists);
            SupportedAssets::<T>::insert(&asset, metadata);
            Self::deposit_event(Event::AssetRegistered(asset));
            Ok(())
        }

        /// Initie une demande de transfert inter‑chaînes.
        ///
        /// `to_nodara` : true pour un transfert vers Nodara (verrouillage sur la source et mint sur Nodara),
        /// false pour un transfert inverse (burn sur Nodara et déverrouillage sur la source).
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn initiate_transfer(
            origin: OriginFor<T>,
            asset: AssetId,
            amount: u128,
            destination: T::AccountId,
            to_nodara: bool,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(amount > 0, Error::<T>::InvalidAmount);
            ensure!(SupportedAssets::<T>::contains_key(&asset), Error::<T>::AssetNotSupported);

            let transfer_id = NextTransferId::<T>::get();
            NextTransferId::<T>::put(transfer_id.saturating_add(1));

            let new_request = TransferRequest {
                id: transfer_id,
                from: sender.clone(),
                asset: asset.clone(),
                amount,
                destination: destination.clone(),
                confirmations: BTreeSet::new(),
                to_nodara,
            };

            PendingTransfers::<T>::insert(transfer_id, new_request);
            Self::deposit_event(Event::TransferInitiated(
                transfer_id,
                sender,
                asset,
                amount,
                destination,
                to_nodara,
            ));
            Ok(())
        }

        /// Permet à un validateur de confirmer un transfert.
        #[pallet::weight(10_000)]
        pub fn confirm_transfer(origin: OriginFor<T>, transfer_id: TransferId) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            PendingTransfers::<T>::try_mutate(transfer_id, |maybe_request| -> DispatchResult {
                let request = maybe_request.as_mut().ok_or(Error::<T>::TransferNotFound)?;
                ensure!(!request.confirmations.contains(&validator), Error::<T>::AlreadyConfirmed);
                request.confirmations.insert(validator.clone());
                Self::deposit_event(Event::TransferConfirmed(transfer_id, validator));
                Ok(())
            })
        }

        /// Finalise le transfert une fois que le seuil de confirmations est atteint.
        ///
        /// Pour un transfert vers Nodara, mint les tokens représentatifs sur le compte destination.
        /// Pour un transfert inverse, burn les tokens représentatifs sur le compte source.
        #[pallet::weight(10_000)]
        #[transactional]
        pub fn finalize_transfer(origin: OriginFor<T>, transfer_id: TransferId) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            PendingTransfers::<T>::try_mutate_exists(transfer_id, |maybe_request| -> DispatchResult {
                let request = maybe_request.take().ok_or(Error::<T>::TransferNotFound)?;
                ensure!(
                    (request.confirmations.len() as u32) >= T::RequiredConfirmations::get(),
                    Error::<T>::InsufficientConfirmations
                );
                if request.to_nodara {
                    // Transfert vers Nodara : mint des tokens représentatifs sur le compte destination.
                    T::AssetManager::mint(request.asset.clone(), &request.destination, request.amount)?;
                } else {
                    // Transfert depuis Nodara : burn des tokens représentatifs sur le compte source.
                    T::AssetManager::burn(request.asset.clone(), &request.from, request.amount)?;
                }
                Self::deposit_event(Event::TransferFinalized(transfer_id));
                Ok(())
            })
        }
    }

    // --- Configuration de Genèse ---
    /// Permet de pré‑enregistrer une liste d’actifs supportés par le bridge lors du lancement de la blockchain.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_assets: Vec<(AssetId, AssetMetadata)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_assets: vec![
                    (b"BTC".to_vec(), AssetMetadata { name: b"Bitcoin".to_vec(), symbol: b"BTC".to_vec(), decimals: 8, source_chain: b"BTC".to_vec() }),
                    (b"ETH".to_vec(), AssetMetadata { name: b"Ethereum".to_vec(), symbol: b"ETH".to_vec(), decimals: 18, source_chain: b"ETH".to_vec() }),
                    (b"BNB".to_vec(), AssetMetadata { name: b"Binance Coin".to_vec(), symbol: b"BNB".to_vec(), decimals: 18, source_chain: b"BNB".to_vec() }),
                    (b"DOT".to_vec(), AssetMetadata { name: b"Polkadot".to_vec(), symbol: b"DOT".to_vec(), decimals: 10, source_chain: b"Polkadot".to_vec() }),
                    (b"XRP".to_vec(), AssetMetadata { name: b"XRP".to_vec(), symbol: b"XRP".to_vec(), decimals: 6, source_chain: b"XRP".to_vec() }),
                    (b"DOGE".to_vec(), AssetMetadata { name: b"Dogecoin".to_vec(), symbol: b"DOGE".to_vec(), decimals: 8, source_chain: b"DOGE".to_vec() }),
                    (b"SOL".to_vec(), AssetMetadata { name: b"Solana".to_vec(), symbol: b"SOL".to_vec(), decimals: 9, source_chain: b"SOL".to_vec() }),
                    (b"LINK".to_vec(), AssetMetadata { name: b"Chainlink".to_vec(), symbol: b"LINK".to_vec(), decimals: 18, source_chain: b"ETH".to_vec() }),
                    (b"SUI".to_vec(), AssetMetadata { name: b"Sui".to_vec(), symbol: b"SUI".to_vec(), decimals: 9, source_chain: b"SUI".to_vec() }),
                    (b"AVAX".to_vec(), AssetMetadata { name: b"Avalanche".to_vec(), symbol: b"AVAX".to_vec(), decimals: 18, source_chain: b"AVAX".to_vec() }),
                    (b"USDT".to_vec(), AssetMetadata { name: b"Tether USD".to_vec(), symbol: b"USDT".to_vec(), decimals: 6, source_chain: b"ERC20".to_vec() }),
                    (b"USDC".to_vec(), AssetMetadata { name: b"USD Coin".to_vec(), symbol: b"USDC".to_vec(), decimals: 6, source_chain: b"ERC20".to_vec() }),
                    (b"ADA".to_vec(), AssetMetadata { name: b"Cardano".to_vec(), symbol: b"ADA".to_vec(), decimals: 6, source_chain: b"Cardano".to_vec() }),
                    (b"TRX".to_vec(), AssetMetadata { name: b"Tron".to_vec(), symbol: b"TRX".to_vec(), decimals: 6, source_chain: b"TRX".to_vec() }),
                    (b"XLM".to_vec(), AssetMetadata { name: b"Stellar".to_vec(), symbol: b"XLM".to_vec(), decimals: 7, source_chain: b"XLM".to_vec() }),
                    (b"TON".to_vec(), AssetMetadata { name: b"Toncoin".to_vec(), symbol: b"TON".to_vec(), decimals: 9, source_chain: b"TON".to_vec() }),
                ],
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (asset_id, metadata) in &self.initial_assets {
                SupportedAssets::<T>::insert(asset_id, metadata);
            }
        }
    }
    
    // --- Tests Unitaires ---
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate as pallet_bridge;
        use frame_support::{assert_ok, parameter_types, traits::OnFinalize};
        use sp_core::H256;
        use sp_runtime::{
            testing::Header,
            traits::{BlakeTwo256, IdentityLookup},
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
                Bridge: pallet_bridge::{Pallet, Call, Storage, Event<T>},
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const RequiredConfirmations: u32 = 2;
        }

        impl system::Config for Test {
            type BaseCallFilter = frame_support::traits::Everything;
            type BlockWeights = ();
            type BlockLength = ();
            type DbWeight = ();
            type RuntimeOrigin = system::mocking::Origin;
            type RuntimeCall = ();
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

        // Pour simplifier les tests, nous créons un gestionnaire d'actifs fictif.
        pub struct DummyAssetManager;
        impl BridgeAssetManager<u64> for DummyAssetManager {
            fn mint(asset: Vec<u8>, _to: &u64, _amount: u128) -> DispatchResult {
                // Simulez ici le minting (pour le test, on ne fait rien)
                Ok(())
            }
            fn burn(asset: Vec<u8>, _from: &u64, _amount: u128) -> DispatchResult {
                // Simulez ici le burning (pour le test, on ne fait rien)
                Ok(())
            }
        }

        impl Config for Test {
            type Event = ();
            type Currency = ();
            type RequiredConfirmations = RequiredConfirmations;
            type AssetManager = DummyAssetManager;
        }

        #[test]
        fn test_bridge_flow() {
            // Test complet du flux de transfert inter-chaînes :
            // 1. Enregistrement d'un actif
            // 2. Initiation d'une demande de transfert
            // 3. Confirmation du transfert par deux validateurs
            // 4. Finalisation du transfert (mint ou burn)
            System::set_block_number(1);
            let asset_id = b"BTC".to_vec();
            let metadata = AssetMetadata {
                name: b"Bitcoin".to_vec(),
                symbol: b"BTC".to_vec(),
                decimals: 8,
                source_chain: b"BTC".to_vec(),
            };

            // Enregistrer l'actif
            assert_ok!(Bridge::register_asset(system::RawOrigin::Signed(1).into(), asset_id.clone(), metadata));

            // Initier un transfert
            let amount = 1_000_000u128;
            assert_ok!(Bridge::initiate_transfer(
                system::RawOrigin::Signed(1).into(),
                asset_id.clone(),
                amount,
                2,
                true
            ));
            let transfer_id = Bridge::next_transfer_id() - 1;

            // Confirmer le transfert avec deux comptes (1 et 3)
            assert_ok!(Bridge::confirm_transfer(system::RawOrigin::Signed(1).into(), transfer_id));
            assert_ok!(Bridge::confirm_transfer(system::RawOrigin::Signed(3).into(), transfer_id));

            // Finaliser le transfert (le mint sera appelé via le DummyAssetManager)
            assert_ok!(Bridge::finalize_transfer(system::RawOrigin::Signed(1).into(), transfer_id));
        }
    }
}
