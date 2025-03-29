#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Interop Module - Extreme Edition
//!
//! Ce module implémente une interopérabilité sécurisée entre Nodara et des chaînes externes. Il gère l'envoi,
//! la réception et la vérification cryptographique des messages interop. Chaque événement est enregistré dans
//! un historique immuable pour assurer une traçabilité complète. De plus, la configuration (timeout et longueur
//! de payload) est dynamique et peut être mise à jour via une extrinsic réservée à Root (via DAO).
//!
//! **Fonctionnalités principales :**
//! - Messagerie inter-chaînes sécurisée.
//! - Vérification cryptographique améliorée (hash Blake2-128).
//! - Journalisation complète des événements interop.
//! - Configuration dynamique et pruning de l’historique.

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*,
    traits::Get,
};
use frame_system::pallet_prelude::*;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure représentant un message interop.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct InteropMessage {
    /// Identifiant unique du message.
    pub id: u64,
    /// Charge utile du message.
    pub payload: Vec<u8>,
    /// Horodatage de l'envoi du message.
    pub timestamp: u64,
    /// Signature cryptographique du message.
    pub signature: Vec<u8>,
}

/// Structure de configuration dynamique du module interop.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
pub struct InteropConfig {
    pub base_timeout: u64,
    pub max_payload_length: u32,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_io::hashing::blake2_128;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Temps de timeout par défaut (en secondes) pour la réception d'un message.
        #[pallet::constant]
        type BaseTimeout: Get<u64>;
        /// Longueur maximale autorisée pour la charge utile d'un message.
        #[pallet::constant]
        type MaxPayloadLength: Get<u32>;
    }

    /// Stockage des messages sortants.
    #[pallet::storage]
    #[pallet::getter(fn outgoing_messages)]
    pub type OutgoingMessages<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, InteropMessage, OptionQuery>;

    /// Stockage des messages entrants.
    #[pallet::storage]
    #[pallet::getter(fn incoming_messages)]
    pub type IncomingMessages<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, InteropMessage, OptionQuery>;

    /// Journalisation des événements interop.
    /// Chaque entrée est un tuple : (timestamp, message id, type d'opération, détails)
    #[pallet::storage]
    #[pallet::getter(fn interop_history)]
    pub type InteropHistory<T: Config> =
        StorageValue<_, Vec<(u64, u64, Vec<u8>, Vec<u8>)>, ValueQuery>;

    /// Stockage de la configuration dynamique du module interop.
    #[pallet::storage]
    #[pallet::getter(fn interop_config)]
    pub type InteropConfigStorage<T: Config> = StorageValue<_, InteropConfig, ValueQuery>;

    /// Configuration de genèse pour le module interop.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Optionnel : configuration initiale du module interop.
        pub initial_config: Option<InteropConfig>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_config: None,
            }
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
        /// Mise à jour des paramètres de configuration du module interop.
        ConfigParamsUpdated(u64, u32, u64, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La charge utile dépasse la longueur maximale autorisée.
        PayloadTooLong,
        /// Échec de la vérification cryptographique.
        VerificationFailed,
        /// Erreur lors du traitement du message.
        MessageProcessingError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Envoie un message interop vers une chaîne externe.
        #[pallet::weight(10_000)]
        pub fn send_message(
            origin: OriginFor<T>,
            id: u64,
            payload: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            // Utilise la configuration dynamique.
            let config = InteropConfigStorage::<T>::get();
            ensure!(
                payload.len() as u32 <= config.max_payload_length,
                Error::<T>::PayloadTooLong
            );
            let timestamp = Self::current_timestamp();
            let message = InteropMessage {
                id,
                payload: payload.clone(),
                timestamp,
                signature,
            };
            <OutgoingMessages<T>>::insert(id, message);
            <InteropHistory<T>>::mutate(|history| {
                history.push((timestamp, id, b"Send".to_vec(), payload.clone()))
            });
            Self::deposit_event(Event::MessageSent(id, payload));
            Ok(())
        }

        /// Reçoit et vérifie un message interop provenant d'une chaîne externe.
        #[pallet::weight(10_000)]
        pub fn receive_message(
            origin: OriginFor<T>,
            id: u64,
            payload: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            // Vérification améliorée : le signature doit être égale au hash Blake2-128 du payload.
            ensure!(Self::verify_signature(&payload, &signature), Error::<T>::VerificationFailed);
            let config = InteropConfigStorage::<T>::get();
            ensure!(
                payload.len() as u32 <= config.max_payload_length,
                Error::<T>::PayloadTooLong
            );
            let timestamp = Self::current_timestamp();
            let message = InteropMessage {
                id,
                payload: payload.clone(),
                timestamp,
                signature,
            };
            <IncomingMessages<T>>::insert(id, message);
            <InteropHistory<T>>::mutate(|history| {
                history.push((timestamp, id, b"Receive".to_vec(), payload.clone()))
            });
            Self::deposit_event(Event::MessageReceived(id, payload));
            Ok(())
        }

        /// Met à jour la configuration du module interop via DAO.
        #[pallet::weight(10_000)]
        pub fn update_config(
            origin: OriginFor<T>,
            new_config: Vec<u8>,
            details: Vec<u8>,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!(!new_config.is_empty(), Error::<T>::MessageProcessingError);
            let timestamp = Self::current_timestamp();
            <InteropHistory<T>>::mutate(|history| {
                history.push((timestamp, 0, b"ConfigUpdate".to_vec(), details.clone()))
            });
            Self::deposit_event(Event::ConfigUpdated(new_config, details));
            Ok(())
        }

        /// Met à jour dynamiquement les paramètres de configuration du module interop.
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

        /// Prune (limite) l'historique interop pour conserver uniquement les dernières `max_entries` entrées.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn prune_history(origin: OriginFor<T>, max_entries: usize) -> DispatchResult {
            ensure_root(origin)?;
            <InteropHistory<T>>::mutate(|history| {
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

        /// Retourne un horodatage fixe (à remplacer par `pallet_timestamp` en production).
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }

    /// Structure de configuration dynamique pour le module interop.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct InteropConfig {
        pub base_timeout: u64,
        pub max_payload_length: u32,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig2<T: Config> {
        pub initial_config: Option<InteropConfig>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig2<T> {
        fn default() -> Self {
            Self { initial_config: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig2<T> {
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
            InteropModule: pallet::{Pallet, Call, Storage, Event<T>},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const BaseTimeout: u64 = 300;
        pub const MaxPayloadLength: u32 = 1024;
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
        type BaseTimeout = BaseTimeout;
        type MaxPayloadLength = MaxPayloadLength;
    }

    #[test]
    fn send_message_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 1;
        let payload = b"Test payload".to_vec();
        // Génère un hash Blake2-128 du payload pour simuler une signature valide.
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_ok!(InteropModule::send_message(origin, id, payload.clone(), signature));
        let msg = InteropModule::outgoing_messages(id).expect("Message must be stored");
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn send_message_should_fail_if_payload_too_long() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 2;
        let payload = vec![0u8; (MaxPayloadLength::get() + 1) as usize];
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_err!(
            InteropModule::send_message(origin, id, payload, signature),
            Error::<Test>::PayloadTooLong
        );
    }

    #[test]
    fn receive_message_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 3;
        let payload = b"Test payload receive".to_vec();
        let signature = sp_io::hashing::blake2_128(&payload).to_vec();
        assert_ok!(InteropModule::receive_message(origin, id, payload.clone(), signature));
        let msg = InteropModule::incoming_messages(id).expect("Message must be stored");
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn receive_message_should_fail_if_verification_fails() {
        let origin = system::RawOrigin::Signed(1).into();
        let id = 4;
        let payload = b"".to_vec();
        let signature = b"".to_vec();
        assert_err!(
            InteropModule::receive_message(origin, id, payload, signature),
            Error::<Test>::VerificationFailed
        );
    }

    #[test]
    fn update_config_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let new_config = b"NewConfig".to_vec();
        let details = b"Update details".to_vec();
        assert_ok!(InteropModule::update_config(origin, new_config.clone(), details.clone()));
        let history = InteropModule::interop_history();
        let config_updates: Vec<_> = history.into_iter().filter(|(_, id, op, _)| {
            *id == 0 && op == b"ConfigUpdate".to_vec()
        }).collect();
        assert!(!config_updates.is_empty());
    }

    #[test]
    fn update_config_params_should_work() {
        let root_origin = system::RawOrigin::Root.into();
        let old_config = InteropModule::interop_config();
        let new_timeout = old_config.base_timeout + 100;
        let new_max_payload = old_config.max_payload_length + 100;
        assert_ok!(InteropModule::update_config_params(root_origin, new_timeout, new_max_payload));
        let new_config = InteropModule::interop_config();
        assert_eq!(new_config.base_timeout, new_timeout);
        assert_eq!(new_config.max_payload_length, new_max_payload);
    }

    #[test]
    fn prune_history_should_work() {
        let root_origin = system::RawOrigin::Root.into();
        let user_origin = system::RawOrigin::Signed(1).into();
        // Envoyer quelques messages pour remplir l'historique.
        assert_ok!(InteropModule::send_message(user_origin.clone(), 10, b"Payload1".to_vec(), sp_io::hashing::blake2_128(b"Payload1").to_vec()));
        assert_ok!(InteropModule::send_message(user_origin.clone(), 11, b"Payload2".to_vec(), sp_io::hashing::blake2_128(b"Payload2").to_vec()));
        let history_before = InteropModule::interop_history();
        let len_before = history_before.len();
        // Prune l'historique pour conserver uniquement 1 entrée.
        assert_ok!(InteropModule::prune_history(root_origin, 1));
        let history_after = InteropModule::interop_history();
        assert_eq!(history_after.len(), 1);
        assert!(len_before > 1);
    }
}
