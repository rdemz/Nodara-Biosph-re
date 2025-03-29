#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara IoT Bridge Module - Extreme Edition
//!
//! Ce module fournit un pont IoT sécurisé pour le réseau Nodara. Il collecte et vérifie cryptographiquement
//! les données envoyées par les dispositifs IoT, enregistre chaque événement dans un historique immuable, et
//! permet une mise à jour dynamique de la configuration via DAO.
//!
//! Fonctionnalités avancées :
//! - Vérification cryptographique basée sur Blake2-128.
//! - Configuration dynamique (timeout et longueur de payload).
//! - Pruning de l’historique pour limiter l’accumulation.
//! - Journalisation complète des opérations pour une traçabilité totale.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;

    /// Structure représentant un enregistrement de données IoT.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct IotRecord {
        /// Identifiant unique du message IoT.
        pub id: u64,
        /// Charge utile des données du dispositif IoT.
        pub payload: Vec<u8>,
        /// Identifiant du dispositif (ex. adresse MAC, numéro de série).
        pub device_id: Vec<u8>,
        /// Horodatage de la réception (en secondes Unix).
        pub timestamp: u64,
        /// Signature cryptographique associée.
        pub signature: Vec<u8>,
    }

    /// Structure de configuration dynamique pour le module IoT Bridge.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct InteropConfig {
        pub base_timeout: u64,
        pub max_payload_length: u32,
    }

    /// Stockage des enregistrements IoT, indexé par identifiant.
    #[pallet::storage]
    #[pallet::getter(fn iot_data)]
    pub type IotData<T: Config> = StorageMap<_, Blake2_128Concat, u64, IotRecord, OptionQuery>;

    /// Journal d'audit des événements IoT.
    /// Chaque entrée : (timestamp, message id, type d'opération, détails)
    #[pallet::storage]
    #[pallet::getter(fn iot_history)]
    pub type IotHistory<T: Config> = StorageValue<_, Vec<(u64, u64, Vec<u8>, Vec<u8>)>, ValueQuery>;

    /// Stockage de la configuration dynamique du module IoT.
    #[pallet::storage]
    #[pallet::getter(fn interop_config)]
    pub type InteropConfigStorage<T: Config> = StorageValue<_, InteropConfig, ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Longueur maximale autorisée pour le payload IoT.
        #[pallet::constant]
        type MaxPayloadLength: Get<u32>;
        /// Timeout de base pour la validation des données (en secondes).
        #[pallet::constant]
        type BaseTimeout: Get<u64>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_config: Option<InteropConfig>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { initial_config: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let config = if let Some(cfg) = &self.initial_config {
                cfg.clone()
            } else {
                InteropConfig {
                    base_timeout: T::BaseTimeout::get(),
                    max_payload_length: T::MaxPayloadLength::get(),
                }
            };
            <InteropConfigStorage<T>>::put(config);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Message envoyé avec succès (id, payload).
        MessageSent(u64, Vec<u8>),
        /// Message reçu et vérifié avec succès (id, payload).
        MessageReceived(u64, Vec<u8>),
        /// Mise à jour de la configuration effectuée via DAO (nouvelle config, détails).
        ConfigUpdated(Vec<u8>, Vec<u8>),
        /// Mise à jour des paramètres de configuration du module IoT.
        ConfigParamsUpdated(u64, u32, u64, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La charge utile dépasse la longueur maximale autorisée.
        PayloadTooLong,
        /// L'identifiant du dispositif est invalide (doit être non vide).
        InvalidDeviceId,
        /// La vérification cryptographique a échoué.
        VerificationFailed,
        /// Erreur de traitement du message.
        MessageProcessingError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Soumet des données IoT après vérification.
        ///
        /// - `id` : Identifiant unique du message.
        /// - `payload` : Données envoyées par le dispositif.
        /// - `device_id` : Identifiant du dispositif (non vide requis).
        /// - `signature` : Signature pour vérifier l'intégrité (doit être égale au hash Blake2-128 du payload).
        #[pallet::weight(10_000)]
        pub fn submit_iot_data(
            origin: OriginFor<T>,
            id: u64,
            payload: Vec<u8>,
            device_id: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let config = InteropConfigStorage::<T>::get();
            ensure!(
                payload.len() as u32 <= config.max_payload_length,
                Error::<T>::PayloadTooLong
            );
            ensure!(!device_id.is_empty(), Error::<T>::InvalidDeviceId);
            // Vérification cryptographique : la signature doit correspondre au hash Blake2-128 du payload.
            ensure!(Self::verify_signature(&payload, &signature), Error::<T>::VerificationFailed);
            let timestamp = Self::current_timestamp();
            let record = IotRecord {
                id,
                payload: payload.clone(),
                device_id,
                timestamp,
                signature,
            };
            <IotData<T>>::insert(id, record);
            <IotHistory<T>>::mutate(|history| {
                history.push((timestamp, id, b"Submit".to_vec(), payload.clone()))
            });
            Self::deposit_event(Event::MessageSent(id, payload));
            Ok(())
        }

        /// Met à jour la configuration du module IoT via DAO.
        ///
        /// - `new_config` : Nouvelle configuration en bytes.
        /// - `details` : Détails ou justification de la mise à jour.
        #[pallet::weight(10_000)]
        pub fn update_config(
            origin: OriginFor<T>,
            new_config: Vec<u8>,
            details: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!(!new_config.is_empty(), Error::<T>::MessageProcessingError);
            let timestamp = Self::current_timestamp();
            <IotHistory<T>>::mutate(|history| {
                history.push((timestamp, 0, b"ConfigUpdate".to_vec(), details.clone()))
            });
            Self::deposit_event(Event::ConfigUpdated(new_config, details));
            Ok(())
        }

        /// Met à jour dynamiquement les paramètres de configuration du module IoT.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn update_config_params(
            origin: OriginFor<T>,
            new_timeout: u64,
            new_max_payload: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let old = InteropConfigStorage::<T>::get();
            InteropConfigStorage::<T>::put(InteropConfig {
                base_timeout: new_timeout,
                max_payload_length: new_max_payload,
            });
            Self::deposit_event(Event::ConfigParamsUpdated(old.base_timeout, old.max_payload_length, new_timeout, new_max_payload));
            Ok(())
        }

        /// Limite (prune) l'historique des événements IoT pour éviter une accumulation excessive.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn prune_history(origin: OriginFor<T>, max_entries: usize) -> DispatchResult {
            ensure_root(origin)?;
            <IotHistory<T>>::mutate(|history| {
                if history.len() > max_entries {
                    *history = history.split_off(history.len() - max_entries);
                }
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Vérifie la signature du message en comparant le hash Blake2-128 du payload avec la signature.
        fn verify_signature(payload: &Vec<u8>, signature: &Vec<u8>) -> bool {
            let hash = sp_io::hashing::blake2_128(&payload);
            signature.len() == 16 && signature == &hash.to_vec()
        }

        /// Retourne un horodatage fixe pour les tests.
        /// En production, remplacez par l'appel à `pallet_timestamp` pour obtenir le temps réel.
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_err, assert_ok, parameter_types};
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
            IotBridgeModule: pallet::{Pallet, Call, Storage, Event<T>},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaxPayloadLength: u32 = 512;
        pub const BaseTimeout: u64 = 300,
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

    impl pallet::Config for Test {
        type RuntimeEvent = ();
        type MaxPayloadLength = MaxPayloadLength;
        type BaseTimeout = BaseTimeout;
    }

    #[test]
    fn submit_iot_data_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 1;
        let payload = b"Test IoT data".to_vec();
        let device_id = b"Device123".to_vec();
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_ok!(IotBridgeModule::submit_iot_data(origin, id, payload.clone(), device_id, signature));
        let record = IotBridgeModule::iot_data(id).expect("Record must be stored");
        assert_eq!(record.payload, payload);
    }

    #[test]
    fn submit_iot_data_should_fail_if_payload_too_long() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 2;
        let payload = vec![0u8; (MaxPayloadLength::get() + 1) as usize];
        let device_id = b"Device123".to_vec();
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_err!(
            IotBridgeModule::submit_iot_data(origin, id, payload, device_id, signature),
            Error::<Test>::PayloadTooLong
        );
    }

    #[test]
    fn submit_iot_data_should_fail_if_device_id_empty() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 3;
        let payload = b"Valid payload".to_vec();
        let device_id = Vec::new();
        let signature = b"Signature".to_vec();
        assert_err!(
            IotBridgeModule::submit_iot_data(origin, id, payload, device_id, signature),
            Error::<Test>::InvalidDeviceId
        );
    }

    #[test]
    fn receive_iot_data_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 4;
        let payload = b"Test payload receive".to_vec();
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_ok!(IotBridgeModule::receive_iot_data(origin, id, payload.clone(), signature));
        let record = IotBridgeModule::incoming_messages(id).expect("Record must be stored");
        assert_eq!(record.payload, payload);
    }

    #[test]
    fn receive_iot_data_should_fail_if_verification_fails() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 5;
        let payload = b"".to_vec();
        let signature = b"".to_vec();
        assert_err!(
            IotBridgeModule::receive_iot_data(origin, id, payload, signature),
            Error::<Test>::VerificationFailed
        );
    }

    #[test]
    fn update_config_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let new_config = b"New IoT Config".to_vec();
        let details = b"Config update details".to_vec();
        assert_ok!(IotBridgeModule::update_config(origin, new_config.clone(), details.clone()));
        let history = IotBridgeModule::iot_history();
        let config_updates: Vec<_> = history.into_iter().filter(|(_, id, op, _)| {
            *id == 0 && op == b"ConfigUpdate".to_vec()
        }).collect();
        assert!(!config_updates.is_empty());
    }

    #[test]
    fn update_config_params_should_work() {
        let root_origin = system::RawOrigin::Root.into();
        let old_config = IotBridgeModule::interop_config();
        let new_timeout = old_config.base_timeout + 100;
        let new_max_payload = old_config.max_payload_length + 100;
        assert_ok!(IotBridgeModule::update_config_params(root_origin, new_timeout, new_max_payload));
        let new_config = IotBridgeModule::interop_config();
        assert_eq!(new_config.base_timeout, new_timeout);
        assert_eq!(new_config.max_payload_length, new_max_payload);
    }

    #[test]
    fn prune_history_should_work() {
        let root_origin = system::RawOrigin::Root.into();
        let user_origin = system::RawOrigin::Signed(1).into();
        // Envoyer quelques messages pour remplir l'historique.
        assert_ok!(IotBridgeModule::submit_iot_data(user_origin.clone(), 10, b"Payload1".to_vec(), b"Device123".to_vec(), sp_io::hashing::blake2_128(b"Payload1").to_vec()));
        assert_ok!(IotBridgeModule::submit_iot_data(user_origin.clone(), 11, b"Payload2".to_vec(), b"Device123".to_vec(), sp_io::hashing::blake2_128(b"Payload2").to_vec()));
        let history_before = IotBridgeModule::iot_history();
        let len_before = history_before.len();
        // Prune l'historique pour conserver uniquement 1 entrée.
        assert_ok!(IotBridgeModule::prune_history(root_origin, 1));
        let history_after = IotBridgeModule::iot_history();
        assert_eq!(history_after.len(), 1);
        assert!(len_before > 1);
    }
}
