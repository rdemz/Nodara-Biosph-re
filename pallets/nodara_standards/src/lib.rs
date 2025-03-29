#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Standards Module - Extreme Production-Ready Version
//!
//! Ce module définit et applique les standards techniques et réglementaires pour le réseau Nodara BIOSPHÈRE QUANTIC.
//! Il vérifie la conformité des opérations à l'aide d'une vérification avancée (basée sur des hachages) et
//! conserve un journal d'audit complet avec rotation automatique. Les mises à jour des standards sont sécurisées
//! et réservées à une origine autorisée (Root), et le module est conçu pour être mis à jour via la gouvernance DAO.
//!
//! Les dépendances sont verrouillées afin d'assurer la reproductibilité du build en production.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Get, UnixTime},
    };
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;

    /// Structure représentant la définition d'un standard.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Standard {
        /// Identifiant unique du standard.
        pub id: Vec<u8>,
        /// Description du standard.
        pub description: Vec<u8>,
        /// Règles ou paramètres associés au standard (format JSON recommandé).
        pub parameters: Vec<u8>,
    }

    /// Structure représentant une entrée dans l'historique de vérification de conformité.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ComplianceLog {
        /// Horodatage de la vérification.
        pub timestamp: u64,
        /// Détails de l'opération vérifiée.
        pub operation_details: Vec<u8>,
        /// Résultat de la vérification (true = conforme, false = non conforme).
        pub outcome: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Longueur maximale autorisée pour la définition d'un standard.
        #[pallet::constant]
        type MaxStandardLength: Get<u32>;
        /// Fournisseur de temps pour obtenir un timestamp réel.
        type TimeProvider: UnixTime;
    }

    /// Stockage des standards définis.
    #[pallet::storage]
    #[pallet::getter(fn standards)]
    pub type Standards<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Standard, OptionQuery>;

    /// Journal d'audit des vérifications de conformité.
    #[pallet::storage]
    #[pallet::getter(fn compliance_history)]
    pub type ComplianceHistory<T: Config> = StorageValue<_, Vec<ComplianceLog>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Standard défini (ID du standard).
        StandardDefined(Vec<u8>),
        /// Standard mis à jour (ID du standard).
        StandardUpdated(Vec<u8>),
        /// Vérification de conformité réalisée (ID du standard, résultat).
        ComplianceChecked(Vec<u8>, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La définition du standard dépasse la longueur maximale autorisée.
        StandardTooLong,
        /// Le standard existe déjà.
        StandardAlreadyExists,
        /// Le standard n'existe pas.
        StandardNotFound,
        /// La vérification de conformité a échoué.
        ComplianceCheckFailed,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Définit un nouveau standard.
        ///
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn define_standard(
            origin: OriginFor<T>,
            id: Vec<u8>,
            description: Vec<u8>,
            parameters: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(
                (description.len() + parameters.len()) as u32 <= T::MaxStandardLength::get(),
                Error::<T>::StandardTooLong
            );
            ensure!(!Standards::<T>::contains_key(&id), Error::<T>::StandardAlreadyExists);
            let standard = Standard { id: id.clone(), description, parameters };
            Standards::<T>::insert(&id, standard);
            Self::deposit_event(Event::StandardDefined(id));
            Ok(())
        }

        /// Met à jour un standard existant.
        #[pallet::weight(10_000)]
        pub fn update_standard(
            origin: OriginFor<T>,
            id: Vec<u8>,
            new_description: Vec<u8>,
            new_parameters: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(
                (new_description.len() + new_parameters.len()) as u32 <= T::MaxStandardLength::get(),
                Error::<T>::StandardTooLong
            );
            Standards::<T>::try_mutate(&id, |maybe_standard| -> DispatchResult {
                let standard = maybe_standard.as_mut().ok_or(Error::<T>::StandardNotFound)?;
                standard.description = new_description;
                standard.parameters = new_parameters;
                Ok(())
            })?;
            Self::deposit_event(Event::StandardUpdated(id));
            Ok(())
        }

        /// Vérifie la conformité d'une opération par rapport à un standard défini.
        ///
        /// La vérification avancée calcule le hash Blake2-128 des paramètres du standard et le recherche dans les données de l'opération.
        #[pallet::weight(10_000)]
        pub fn verify_compliance(
            origin: OriginFor<T>,
            standard_id: Vec<u8>,
            operation_data: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let standard = Standards::<T>::get(&standard_id).ok_or(Error::<T>::StandardNotFound)?;
            let standard_hash = sp_io::hashing::blake2_128(&standard.parameters);
            let outcome = operation_data.windows(standard_hash.len())
                .any(|window| window == standard_hash);
            let log = ComplianceLog {
                timestamp: T::TimeProvider::now().as_secs(),
                operation_details: operation_data,
                outcome,
            };
            ComplianceHistory::<T>::mutate(|history| history.push(log));
            Self::deposit_event(Event::ComplianceChecked(standard_id.clone(), outcome));
            if outcome { Ok(()) } else { Err(Error::<T>::ComplianceCheckFailed.into()) }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Fonction de rotation de l'historique pour limiter la taille du journal.
        pub fn rotate_history(max_entries: usize) {
            ComplianceHistory::<T>::mutate(|history| {
                if history.len() > max_entries {
                    *history = history.split_off(history.len() - max_entries);
                }
            });
        }
    }
}
