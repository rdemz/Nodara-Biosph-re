#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara ID Module - Version Complète et Verrouillée
//!
//! Ce module gère l'enregistrement et la mise à jour des identités décentralisées pour le réseau Nodara BIOSPHÈRE QUANTIC.
//! Il inclut la gestion des erreurs, des événements, et des appels extrinsics pour enregistrer et mettre à jour l'identité d'un compte.
//! La sérialisation est assurée par `parity-scale-codec` et les métadonnées par `scale-info`.
//!
//! **Note de déploiement :** Le timestamp utilisé ici est une valeur fixe pour les tests. En production, remplacez
//! cette fonction par l'appel au `pallet_timestamp` afin d'obtenir un timestamp réel.
//!
//! ## Fonctionnalités
//! - Enregistrement d'une identité avec des détails KYC et statut de vérification par défaut.
//! - Mise à jour d'une identité existante avec conservation de l'historique.
//! - Stockage de l'historique des modifications pour audit et traçabilité.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::{Decode, Encode};
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;

    /// Structure représentant les données d'identité d'un compte.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct IdentityData {
        /// Détails KYC (peut contenir des données chiffrées).
        pub kyc_details: Vec<u8>,
        /// Statut de vérification de l'identité.
        pub verified: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Valeur par défaut du statut de vérification (true = vérifié).
        #[pallet::constant]
        type DefaultVerification: Get<bool>;
        /// Longueur maximale autorisée pour les détails KYC.
        #[pallet::constant]
        type MaxKycLength: Get<u32>;
    }

    /// Stockage des identités : associe chaque compte à ses données d'identité.
    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, IdentityData, OptionQuery>;

    /// Historique des mises à jour d'identité.
    /// Chaque entrée est un tuple : (timestamp, AccountId, ancien statut, nouveau statut, détails KYC)
    #[pallet::storage]
    #[pallet::getter(fn identity_history)]
    pub type IdentityHistory<T: Config> =
        StorageValue<_, Vec<(u64, T::AccountId, bool, bool, Vec<u8>)>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors de l'enregistrement d'une identité.
        IdentityRegistered(T::AccountId, Vec<u8>, bool),
        /// Événement émis lors de la mise à jour d'une identité.
        IdentityUpdated(T::AccountId, Vec<u8>, bool, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Les détails KYC dépassent la longueur maximale autorisée.
        KycTooLong,
        /// Une identité est déjà enregistrée pour ce compte.
        IdentityAlreadyExists,
        /// Aucune identité trouvée pour ce compte.
        IdentityNotFound,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Enregistre une nouvelle identité pour le compte appelant.
        ///
        /// - **origin**: Le compte qui s'enregistre.
        /// - **kyc_details**: Détails KYC sous forme d'octets.
        #[pallet::weight(10_000)]
        pub fn register_identity(
            origin: OriginFor<T>,
            kyc_details: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                kyc_details.len() as u32 <= T::MaxKycLength::get(),
                Error::<T>::KycTooLong
            );
            ensure!(
                !Identities::<T>::contains_key(&who),
                Error::<T>::IdentityAlreadyExists
            );
            let identity = IdentityData {
                kyc_details: kyc_details.clone(),
                verified: T::DefaultVerification::get(),
            };
            <Identities<T>>::insert(&who, identity);
            let timestamp = Self::current_timestamp();
            <IdentityHistory<T>>::mutate(|history| {
                history.push((timestamp, who.clone(), false, T::DefaultVerification::get(), kyc_details.clone()))
            });
            Self::deposit_event(Event::IdentityRegistered(who, kyc_details, T::DefaultVerification::get()));
            Ok(())
        }

        /// Met à jour l'identité du compte appelant.
        ///
        /// - **origin**: Le compte qui met à jour son identité.
        /// - **new_kyc_details**: Nouveaux détails KYC.
        /// - **new_verified**: Nouveau statut de vérification.
        #[pallet::weight(10_000)]
        pub fn update_identity(
            origin: OriginFor<T>,
            new_kyc_details: Vec<u8>,
            new_verified: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                new_kyc_details.len() as u32 <= T::MaxKycLength::get(),
                Error::<T>::KycTooLong
            );
            Identities::<T>::try_mutate(&who, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                let prev_verified = identity.verified;
                identity.kyc_details = new_kyc_details.clone();
                identity.verified = new_verified;
                let timestamp = Self::current_timestamp();
                <IdentityHistory<T>>::mutate(|history| {
                    history.push((timestamp, who.clone(), prev_verified, new_verified, new_kyc_details.clone()))
                });
                Self::deposit_event(Event::IdentityUpdated(who, new_kyc_details, prev_verified, new_verified));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Retourne un timestamp fixe.
        ///
        /// **Attention :** Cette fonction utilise une valeur statique pour simplifier les tests.
        /// En production, intégrez le `pallet_timestamp` pour obtenir le temps réel.
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}

