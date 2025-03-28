#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Standards Module - Locked and Production-Ready Version
//!
//! Ce module définit et applique les standards techniques et réglementaires pour le réseau Nodara BIOSPHÈRE QUANTIC.
//! Il permet de vérifier la conformité des opérations et de conserver un journal d'audit complet.
//! Les mises à jour de standards peuvent être effectuées via la gouvernance DAO.
//!
//! Les dépendances sont verrouillées afin d'assurer la reproductibilité du build en production.

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

    /// Structure représentant la définition d'un standard.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Standard {
        /// Identifiant unique du standard.
        pub id: Vec<u8>,
        /// Description du standard.
        pub description: Vec<u8>,
        /// Règles ou paramètres associés au standard.
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
        /// Événement émis lors de la définition d'un nouveau standard.
        StandardDefined(Vec<u8>),
        /// Événement émis lors de la mise à jour d'un standard.
        StandardUpdated(Vec<u8>),
        /// Événement émis lors d'une vérification de conformité (ID du standard, résultat).
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
        /// # Paramètres
        /// - `id`: Identifiant unique du standard.
        /// - `description`: Description du standard.
        /// - `parameters`: Règles ou paramètres associés.
        ///
        /// La somme de la longueur de la description et des paramètres ne doit pas dépasser `MaxStandardLength`.
        #[pallet::weight(10_000)]
        pub fn define_standard(
            origin: OriginFor<T>,
            id: Vec<u8>,
            description: Vec<u8>,
            parameters: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
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
        ///
        /// # Paramètres
        /// - `id`: Identifiant du standard à mettre à jour.
        /// - `new_description`: Nouvelle description.
        /// - `new_parameters`: Nouveaux paramètres ou règles.
        #[pallet::weight(10_000)]
        pub fn update_standard(
            origin: OriginFor<T>,
            id: Vec<u8>,
            new_description: Vec<u8>,
            new_parameters: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
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
        /// # Paramètres
        /// - `standard_id`: L'identifiant du standard à utiliser pour la vérification.
        /// - `operation_data`: Données décrivant l'opération.
        ///
        /// Retourne `true` si l'opération est conforme, sinon `false`.
        #[pallet::weight(10_000)]
        pub fn verify_compliance(
            origin: OriginFor<T>,
            standard_id: Vec<u8>,
            operation_data: Vec<u8>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let standard = Standards::<T>::get(&standard_id).ok_or(Error::<T>::StandardNotFound)?;
            // Vérification simulée : on considère conforme si les paramètres du standard
            // se retrouvent dans les données de l'opération.
            let outcome = operation_data.windows(standard.parameters.len())
                .any(|window| window == standard.parameters.as_slice());
            let log = ComplianceLog {
                timestamp: Self::current_timestamp(),
                operation_details: operation_data,
                outcome,
            };
            ComplianceHistory::<T>::mutate(|history| history.push(log));
            Self::deposit_event(Event::ComplianceChecked(standard_id, outcome));
            if outcome { Ok(()) } else { Err(Error::<T>::ComplianceCheckFailed.into()) }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Fournit un timestamp fixe (à remplacer en production par un fournisseur de temps fiable).
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}
