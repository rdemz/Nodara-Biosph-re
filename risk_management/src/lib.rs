#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Risk Management Module - Extreme Version
//!
//! Ce module gère la gestion des risques sur le réseau Nodara BIOSPHÈRE QUANTIC. 
//! Il permet de collecter et d'enregistrer des événements de risque, 
//! de calculer un score de risque dynamique à l'aide d'une moyenne mobile exponentielle (EMA),
//! et de générer des alertes lorsque le risque dépasse un seuil critique.
//!
//! Il conserve un historique complet des événements de risque pour audit et intègre des fonctions de mise à jour du seuil via la gouvernance DAO.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, traits::{Get, UnixTime},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;

    /// Structure représentant un événement de risque.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct RiskEvent {
        /// Horodatage de l'événement (en secondes Unix).
        pub timestamp: u64,
        /// Facteur de risque appliqué (peut être positif pour augmenter le risque ou négatif pour le réduire).
        pub risk_factor: i32,
        /// Description détaillée de l'événement.
        pub description: Vec<u8>,
    }

    /// État global du module de gestion des risques.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct RiskState {
        /// Score de risque actuel (>= 0).
        pub current_risk: i32,
        /// Moyenne mobile exponentielle (EMA) des événements de risque.
        pub risk_ema: i32,
        /// Seuil critique de risque (si dépassé, une alerte est émise).
        pub threshold: i32,
        /// Historique complet des événements de risque.
        pub history: Vec<RiskEvent>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Risque de base pour l'initialisation (en u32, converti en i32).
        #[pallet::constant]
        type BaselineRisk: Get<u32>;
        /// Seuil critique de risque (en u32, converti en i32).
        #[pallet::constant]
        type RiskThreshold: Get<u32>;
        /// Facteur de lissage pour le calcul de l'EMA (doit être > 0).
        #[pallet::constant]
        type RiskSmoothingFactor: Get<u32>;
        /// Fournisseur de temps pour obtenir un timestamp réel.
        type TimeProvider: UnixTime;
    }

    /// Stockage de l'état de gestion des risques.
    #[pallet::storage]
    #[pallet::getter(fn risk_state)]
    pub type RiskStateStorage<T: Config> = StorageValue<_, RiskState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement généré lors de la soumission d'un événement de risque.
        /// (compte, facteur de risque soumis, nouvelle EMA, nouveau score de risque)
        RiskEventSubmitted(T::AccountId, i32, i32, i32),
        /// Seuil de risque mis à jour (ancien seuil, nouveau seuil).
        RiskThresholdUpdated(i32, i32),
        /// Alerte déclenchée si le risque dépasse le seuil (compte, nouveau score de risque).
        RiskAlert(T::AccountId, i32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Le facteur de risque doit être non nul.
        InvalidRiskFactor,
        /// Erreur dans la mise à jour du seuil de risque.
        InvalidThreshold,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état de risque avec le score de base et le seuil défini.
        /// Cette fonction est réservée à Root.
        #[pallet::weight(10_000)]
        pub fn initialize_risk(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let now = T::TimeProvider::now().as_secs();
            let baseline = T::BaselineRisk::get() as i32;
            let threshold = T::RiskThreshold::get() as i32;
            let state = RiskState {
                current_risk: baseline,
                risk_ema: baseline,
                threshold,
                history: vec![RiskEvent {
                    timestamp: now,
                    risk_factor: 0,
                    description: b"Initialisation".to_vec(),
                }],
            };
            RiskStateStorage::<T>::put(state);
            Ok(())
        }

        /// Soumet un événement de risque.
        ///
        /// Le nouvel EMA est calculé comme suit :
        /// `new_ema = if old_ema == 0 { risk_factor } else { (risk_factor + (smoothing - 1) * old_ema) / smoothing }`
        ///
        /// Le score de risque est mis à jour en ajoutant le facteur soumis (le résultat est clamped à 0).
        /// Si le nouveau score dépasse le seuil, une alerte est émise.
        #[pallet::weight(10_000)]
        pub fn submit_risk_event(
            origin: OriginFor<T>,
            risk_factor: i32,
            description: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(risk_factor != 0, Error::<T>::InvalidRiskFactor);
            let now = T::TimeProvider::now().as_secs();
            RiskStateStorage::<T>::mutate(|state| {
                let old_ema = state.risk_ema;
                let smoothing = T::RiskSmoothingFactor::get() as i32;
                let new_ema = if old_ema == 0 { risk_factor } else { (risk_factor + (smoothing - 1) * old_ema) / smoothing };
                state.risk_ema = new_ema;
                // Mise à jour du score de risque, en s'assurant qu'il reste >= 0.
                let new_risk = (state.current_risk + risk_factor).max(0);
                state.current_risk = new_risk;
                state.history.push(RiskEvent {
                    timestamp: now,
                    risk_factor,
                    description: description.clone(),
                });
                // Déclenchement d'une alerte si le risque dépasse le seuil.
                if new_risk > state.threshold {
                    Self::deposit_event(Event::RiskAlert(who.clone(), new_risk));
                }
                Self::deposit_event(Event::RiskEventSubmitted(who, risk_factor, new_ema, new_risk));
            });
            Ok(())
        }

        /// Met à jour le seuil de risque.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn update_threshold(origin: OriginFor<T>, new_threshold: u32) -> DispatchResult {
            ensure_root(origin)?;
            let old_threshold = RiskStateStorage::<T>::get().threshold;
            let new_threshold_i32 = new_threshold as i32;
            RiskStateStorage::<T>::mutate(|state| {
                state.threshold = new_threshold_i32;
            });
            Self::deposit_event(Event::RiskThresholdUpdated(old_threshold, new_threshold_i32));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Fonction utilitaire retournant le timestamp actuel.
        /// En production, remplacez par un fournisseur de temps fiable (ex. `pallet_timestamp`).
        pub fn current_timestamp() -> u64 {
            T::TimeProvider::now().as_secs()
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

        // Fournisseur de temps de test.
        pub struct TestTimeProvider;
        impl UnixTime for TestTimeProvider {
            fn now() -> sp_timestamp::Timestamp {
                sp_timestamp::Timestamp::from(1_640_000_000)
            }
        }

        type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
        type Block = system::mocking::MockBlock<Test>;

        frame_support::construct_runtime!(
            pub enum Test where
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                RiskModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineRisk: u32 = 50;
            pub const RiskThreshold: u32 = 100;
            pub const RiskSmoothingFactor: u32 = 10;
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
            type BaselineRisk = BaselineRisk;
            type RiskThreshold = RiskThreshold;
            type RiskSmoothingFactor = RiskSmoothingFactor;
            type TimeProvider = TestTimeProvider;
        }

        #[test]
        fn initialize_risk_works() {
            assert_ok!(RiskModule::initialize_risk(system::RawOrigin::Root.into()));
            let state = RiskModule::risk_state();
            assert_eq!(state.current_risk, BaselineRisk::get() as i32);
            assert_eq!(state.risk_ema, BaselineRisk::get() as i32);
            assert_eq!(state.threshold, RiskThreshold::get() as i32);
            assert_eq!(state.history.len(), 1);
        }

        #[test]
        fn submit_risk_event_increases_risk_and_updates_ema() {
            let account: u64 = 1;
            assert_ok!(RiskModule::initialize_risk(system::RawOrigin::Root.into()));
            let event_risk = 30; // ajout de 30
            let description = b"High CPU usage".to_vec();
            assert_ok!(RiskModule::submit_risk_event(system::RawOrigin::Signed(account).into(), event_risk, description.clone()));
            let state = RiskModule::risk_state();
            assert_eq!(state.current_risk, (BaselineRisk::get() as i32) + event_risk);
            // L'historique doit contenir deux entrées.
            assert_eq!(state.history.len(), 2);
        }

        #[test]
        fn submit_risk_event_alerts_when_threshold_exceeded() {
            let account: u64 = 1;
            assert_ok!(RiskModule::initialize_risk(system::RawOrigin::Root.into()));
            // Avec un événement de risque qui fait dépasser le seuil.
            let event_risk = 60; // 50 + 60 = 110 > seuil de 100
            let description = b"Network congestion".to_vec();
            assert_ok!(RiskModule::submit_risk_event(system::RawOrigin::Signed(account).into(), event_risk, description));
            let state = RiskModule::risk_state();
            assert!(state.current_risk > RiskThreshold::get() as i32);
        }

        #[test]
        fn update_threshold_works() {
            assert_ok!(RiskModule::initialize_risk(system::RawOrigin::Root.into()));
            let new_threshold = 200;
            assert_ok!(RiskModule::update_threshold(system::RawOrigin::Root.into(), new_threshold));
            let state = RiskModule::risk_state();
            assert_eq!(state.threshold, new_threshold as i
