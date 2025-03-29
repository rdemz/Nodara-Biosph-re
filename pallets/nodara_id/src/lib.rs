#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara ID Module - Version Complète et Verrouillée (Extreme Edition)
//!
//! Ce module gère l'enregistrement et la mise à jour des identités décentralisées pour le réseau Nodara BIOSPHÈRE QUANTIC.
//! Il inclut une gestion avancée des erreurs, des événements et des appels extrinsics pour enregistrer et mettre à jour
//! l'identité d'un compte. Un historique complet des modifications est conservé (avec une fonction de pruning) pour
//! garantir une traçabilité optimale.  
//!
//! **Note de déploiement :** Le timestamp utilisé ici est fixe pour les tests. En production, remplacez cette fonction
//! par un appel au `pallet_timestamp` pour obtenir un temps réel.

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

    /// Structure représentant les données d'identité d'un compte.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct IdentityData {
        /// Détails KYC (doivent être chiffrés en production).
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

    /// Erreurs spécifiques au module d'identité.
    #[pallet::error]
    pub enum Error<T> {
        /// Les détails KYC dépassent la longueur maximale autorisée.
        KycTooLong,
        /// Les détails KYC ne peuvent pas être vides.
        InvalidKycDetails,
        /// Une identité est déjà enregistrée pour ce compte.
        IdentityAlreadyExists,
        /// Aucune identité trouvée pour ce compte.
        IdentityNotFound,
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

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Extrinsics pour gérer l'enregistrement et la mise à jour des identités.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Enregistre une nouvelle identité pour le compte appelant.
        ///
        /// - **origin** : Le compte qui s'enregistre.
        /// - **kyc_details** : Détails KYC sous forme d'octets (doivent être non vides et ne pas dépasser MaxKycLength).
        #[pallet::weight(10_000)]
        pub fn register_identity(
            origin: OriginFor<T>,
            kyc_details: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!kyc_details.is_empty(), Error::<T>::InvalidKycDetails);
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
        /// - **origin** : Le compte qui met à jour son identité.
        /// - **new_kyc_details** : Nouveaux détails KYC (non vides et conformes à la limite).
        /// - **new_verified** : Nouveau statut de vérification.
        #[pallet::weight(10_000)]
        pub fn update_identity(
            origin: OriginFor<T>,
            new_kyc_details: Vec<u8>,
            new_verified: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!new_kyc_details.is_empty(), Error::<T>::InvalidKycDetails);
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

        /// Prune (limite) l'historique des mises à jour d'identité pour éviter une accumulation excessive.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn prune_identity_history(origin: OriginFor<T>, max_entries: u32) -> DispatchResult {
            ensure_root(origin)?;
            IdentityHistory::<T>::mutate(|history| {
                if (history.len() as u32) > max_entries {
                    *history = history.split_off(history.len() - (max_entries as usize));
                }
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Retourne un timestamp fixe.
        /// En production, remplacez par l'appel à `pallet_timestamp` pour obtenir le temps réel.
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Identité enregistrée. (compte, détails KYC, statut de vérification)
        IdentityRegistered(T::AccountId, Vec<u8>, bool),
        /// Identité mise à jour. (compte, nouveaux détails KYC, ancien statut, nouveau statut)
        IdentityUpdated(T::AccountId, Vec<u8>, bool, bool),
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
            IdentityModule: pallet::{Pallet, Call, Storage, Event<T>},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const DefaultVerification: bool = true;
        pub const MaxKycLength: u32 = 256;
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
        type DefaultVerification = DefaultVerification;
        type MaxKycLength = MaxKycLength;
    }

    #[test]
    fn register_identity_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let kyc_details = b"Encrypted KYC Data".to_vec();
        assert_ok!(IdentityModule::register_identity(origin, kyc_details.clone()));
        let identity = IdentityModule::identities(1).expect("L'identité doit être enregistrée");
        assert_eq!(identity.kyc_details, kyc_details);
        assert_eq!(identity.verified, DefaultVerification::get());
        let history = IdentityModule::identity_history();
        assert!(!history.is_empty());
    }

    #[test]
    fn register_identity_should_fail_if_already_exists() {
        let origin = system::RawOrigin::Signed(1).into();
        let kyc_details = b"KYC Data".to_vec();
        assert_ok!(IdentityModule::register_identity(origin.clone(), kyc_details.clone()));
        assert_err!(
            IdentityModule::register_identity(origin, kyc_details),
            Error::<Test>::IdentityAlreadyExists
        );
    }

    #[test]
    fn update_identity_should_work() {
        let origin = system::RawOrigin::Signed(1).into();
        let kyc_details = b"Initial KYC Data".to_vec();
        assert_ok!(IdentityModule::register_identity(origin.clone(), kyc_details));
        let new_details = b"Updated KYC Data".to_vec();
        assert_ok!(IdentityModule::update_identity(system::RawOrigin::Signed(1).into(), new_details.clone(), false));
        let identity = IdentityModule::identities(1).expect("L'identité doit exister");
        assert_eq!(identity.kyc_details, new_details);
        assert_eq!(identity.verified, false);
    }

    #[test]
    fn update_identity_should_fail_if_not_found() {
        let new_details = b"Test".to_vec();
        assert_err!(
            IdentityModule::update_identity(system::RawOrigin::Signed(99).into(), new_details, false),
            Error::<Test>::IdentityNotFound
        );
    }

    #[test]
    fn prune_history_should_work() {
        let root_origin = system::RawOrigin::Root.into();
        let user_origin = system::RawOrigin::Signed(1).into();
        // Enregistrer une identité pour créer des entrées dans l'historique.
        assert_ok!(IdentityModule::register_identity(user_origin.clone(), b"Data".to_vec()));
        // Mettre à jour plusieurs fois pour accumuler l'historique.
        for i in 0..10 {
            let details = format!("Update {}", i).into_bytes();
            assert_ok!(IdentityModule::update_identity(system::RawOrigin::Signed(1).into(), details, false));
        }
        let history_before = IdentityModule::identity_history();
        let len_before = history_before.len() as u32;
        // Prune l'historique pour conserver uniquement 5 entrées.
        assert_ok!(IdentityModule::prune_identity_history(root_origin, 5));
        let history_after = IdentityModule::identity_history();
        assert_eq!(history_after.len() as u32, 5);
        assert!(len_before > 5);
    }
}
