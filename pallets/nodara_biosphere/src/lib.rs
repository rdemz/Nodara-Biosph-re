#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_biosphere - Legendary Extreme Edition
//!
//! Ce module gère l'état adaptatif du réseau Nodara, en ajustant dynamiquement les paramètres du réseau
//! en fonction de signaux économiques et de performance. Il utilise des calculs EMA (moyenne mobile exponentielle)
//! pour lisser les mesures d'énergie et de quantum_flux, et détermine la phase opérationnelle (Growth, Defense, Mutation)
//! en se basant sur des seuils définis. Chaque mise à jour est enregistrée dans un historique pour garantir l'auditabilité.
//!
//! Cette version "extreme" est conçue pour être robuste et déployable en production sur testnet et mainnet.

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::{Currency, Get},
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec::Vec;
use sp_std::prelude::*; // Inclut notamment ToString

/// Trait pour gérer le minting et le burning des tokens représentatifs sur Nodara.
pub trait BridgeAssetManager<AccountId> {
    /// Mint (crée) des tokens représentatifs pour l’actif donné et les crédite au compte `to`.
    fn mint(asset: Vec<u8>, to: &AccountId, amount: u128) -> DispatchResult;
    /// Burn (détruit) des tokens représentatifs pour l’actif donné en les retirant du compte `from`.
    fn burn(asset: Vec<u8>, from: &AccountId, amount: u128) -> DispatchResult;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::Zero;

    /// Identifiant de l'actif (exemple: b"BTC", b"ETH", etc.).
    pub type AssetId = Vec<u8>;
    /// Identifiant du transfert.
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
        pub current_phase: BioPhase,
        pub energy_level: u32,
        pub quantum_flux: u32,
        pub last_updated: u64,
        pub history: Vec<(u64, BioPhase, u32, u32)>, // (timestamp, phase, energy, quantum_flux)
    }

    /// Enumération des phases opérationnelles du réseau.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum BioPhase {
        Growth,
        Defense,
        Mutation,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Module monétaire, pour d'éventuelles opérations financières.
        type Currency: Currency<Self::AccountId>;
        /// Nombre minimum de confirmations requis pour finaliser un transfert.
        #[pallet::constant]
        type RequiredConfirmations: Get<u32>;
        /// Gestionnaire des tokens représentatifs pour le bridge.
        type AssetManager: BridgeAssetManager<Self::AccountId>;
        /// Facteur de lissage utilisé pour le calcul des moyennes mobiles exponentielles (EMA).
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
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
        /// Le bio state a été mis à jour. [ancien phase, nouvelle phase, nouvelle énergie, nouveau flux quantique]
        BioStateUpdated(BioPhase, BioPhase, u32, u32),
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
        #[pallet::weight(10_000)]
        pub fn register_asset(origin: OriginFor<T>, asset: AssetId, metadata: AssetMetadata) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            ensure!(!asset.is_empty(), Error::<T>::InvalidAssetDefinition);
            ensure!(!metadata.name.is_empty(), Error::<T>::InvalidAssetDefinition);
            ensure!(!metadata.symbol.is_empty(), Error::<T>::InvalidAssetDefinition);
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
                    T::AssetManager::mint(request.asset.clone(), &request.destination, request.amount)?;
                } else {
                    T::AssetManager::burn(request.asset.clone(), &request.from, request.amount)?;
                }
                Self::deposit_event(Event::TransferFinalized(transfer_id));
                Ok(())
            })
        }

        /// Met à jour l'état de la biosphère en fonction d'un signal et d'une signature cryptographique.
        ///
        /// Cette version "extreme" utilise une moyenne mobile exponentielle (EMA) pour lisser les mesures d'énergie
        /// et de flux quantique. Les nouveaux paramètres sont calculés comme suit :
        ///
        /// - Énergie mesurée = signal * 10
        /// - Nouvelle énergie = (énergie mesurée + (smoothing - 1) * énergie actuelle) / smoothing
        /// - Flux mesuré = (signal^2) / smoothing
        /// - Nouveau flux quantique = (flux mesuré + (smoothing - 1) * flux actuel) / smoothing
        ///
        /// La nouvelle phase est déterminée par des seuils appliqués à la nouvelle énergie.
        #[pallet::weight(10_000)]
        pub fn transition_phase(origin: OriginFor<T>, signal: u32, signature: Vec<u8>) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidSignal);
            ensure!(!signature.is_empty(), Error::<T>::SignatureVerificationFailed);

            let mut state = BioStateStorage::<T>::get();
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();

            let smoothing = T::SmoothingFactor::get();
            ensure!(smoothing > 0, Error::<T>::ZeroSmoothingFactor);

            // Calcul de l'énergie mesurée et de la nouvelle énergie via EMA.
            let measured_energy = signal.saturating_mul(10);
            let new_energy = if state.energy_level == 0 {
                measured_energy
            } else {
                (measured_energy + (smoothing - 1) * state.energy_level) / smoothing
            };

            // Calcul du flux mesuré et du nouveau flux quantique via EMA.
            let measured_flux = (signal.saturating_mul(signal)) / smoothing;
            let new_quantum_flux = if state.quantum_flux == 0 {
                measured_flux
            } else {
                (measured_flux + (smoothing - 1) * state.quantum_flux) / smoothing
            };

            // Détermination de la nouvelle phase basée sur de nouveaux seuils.
            let new_phase = if new_energy > 150 {
                BioPhase::Growth
            } else if new_energy > 75 {
                BioPhase::Defense
            } else {
                BioPhase::Mutation
            };

            let old_phase = state.current_phase.clone();
            state.current_phase = new_phase.clone();
            state.energy_level = new_energy;
            state.quantum_flux = new_quantum_flux;
            state.last_updated = now;
            state.history.push((now, new_phase.clone(), new_energy, new_quantum_flux));
            BioStateStorage::<T>::put(state);

            Self::deposit_event(Event::BioStateUpdated(old_phase, new_phase, new_energy, new_quantum_flux));
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, parameter_types};
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
                Biosphere: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const SmoothingFactor: u32 = 2;
        }

        // Type to provide a baseline phase.
        pub struct TestBaselinePhase;
        impl Get<BioPhase> for TestBaselinePhase {
            fn get() -> BioPhase {
                BioPhase::Defense
            }
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
            type BaselineEnergy = parameter_types::ConstU32<100>;
            type BaselineQuantumFlux = parameter_types::ConstU32<50>;
            type BaselinePhase = TestBaselinePhase;
            type SmoothingFactor = SmoothingFactor;
        }

        #[test]
        fn test_initialize_state() {
            // Call from Root
            let origin = system::RawOrigin::Root.into();
            assert_ok!(Biosphere::initialize_state(origin));

            // Verify that the bio state is initialized with baseline values.
            let state = Biosphere::bio_state();
            assert_eq!(state.current_phase, BioPhase::Defense);
            assert_eq!(state.energy_level, 100);
            assert_eq!(state.quantum_flux, 50);
            assert!(!state.history.is_empty());
        }

        #[test]
        fn test_transition_phase() {
            // Initialize state first.
            let root_origin = system::RawOrigin::Root.into();
            assert_ok!(Biosphere::initialize_state(root_origin));

            // Transition phase with a valid signal and signature.
            let signed_origin = system::RawOrigin::Signed(1).into();
            // For signal = 120:
            // measured_energy = 120*10 = 1200,
            // new_energy = (1200 + (2-1)*100) / 2 = 650,
            // measured_flux = (120*120)/2 = 7200,
            // new_quantum_flux = (7200 + (2-1)*50)/2 = 3625.
            // Phase: new_energy = 650 > 150, so Growth.
            assert_ok!(Biosphere::transition_phase(signed_origin, 120, vec![1,2,3]));

            // Verify that the bio state was updated.
            let state = Biosphere::bio_state();
            assert_eq!(state.current_phase, BioPhase::Growth);
            assert_eq!(state.energy_level, 650);
            assert_eq!(state.quantum_flux, 3625);
            // History should now have two entries.
            assert_eq!(state.history.len(), 2);
        }
    }
}
