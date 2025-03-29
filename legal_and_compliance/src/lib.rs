#![cfg_attr(not(feature = "std"), no_std)]

//! # Nodara Legal and Compliance Module - Extreme Production-Ready Version
//!
//! Ce module applique des contraintes réglementaires et légales on-chain. Il gère les scores de conformité des comptes,
//! enregistre l'historique complet des mises à jour et permet d'adapter dynamiquement le seuil minimal de conformité
//! via la gouvernance DAO.
//!
//! Fonctionnalités avancées :
//! - Stockage dynamique et mise à jour du seuil de conformité.
//! - Audit logging complet des mises à jour de conformité.
//! - Extraction d'un rapport de conformité pour chaque compte.
//! - Pruning de l'historique pour éviter une accumulation excessive.
//! - Intégration avec un fournisseur de temps pour obtenir des timestamps réels.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Get, UnixTime},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// Structure représentant une entrée dans l'historique de mise à jour de la conformité.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ComplianceLog<T: Config> {
        /// Compte concerné.
        pub account: T::AccountId,
        /// Score de conformité mis à jour.
        pub score: u32,
        /// Horodatage de l'opération.
        pub timestamp: u64,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Niveau minimal de conformité initial pour un compte.
        #[pallet::constant]
        type InitialMinComplianceLevel: Get<u32>;
        /// Fournisseur de temps pour obtenir un timestamp réel.
        type TimeProvider: UnixTime;
    }

    /// Stockage des scores de conformité par compte.
    #[pallet::storage]
    #[pallet::getter(fn compliance_status)]
    pub type ComplianceStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// Historique complet des mises à jour de conformité.
    #[pallet::storage]
    #[pallet::getter(fn compliance_history)]
    pub type ComplianceHistory<T: Config> =
        StorageValue<_, Vec<ComplianceLog<T>>, ValueQuery>;

    /// Stockage du seuil minimal de conformité (modifiable via gouvernance).
    #[pallet::storage]
    #[pallet::getter(fn min_compliance_level)]
    pub type MinComplianceLevelStorage<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mise à jour du score de conformité d'un compte. (compte, nouveau score)
        ComplianceUpdated(T::AccountId, u32),
        /// Seuil minimal de conformité mis à jour. (ancien seuil, nouveau seuil)
        MinComplianceLevelUpdated(u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Le score de conformité soumis est inférieur au seuil minimal.
        ComplianceLevelTooLow,
        /// Aucun enregistrement de conformité trouvé pour le compte.
        ComplianceRecordNotFound,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_min_compliance: u32,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { initial_min_compliance: T::InitialMinComplianceLevel::get() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MinComplianceLevelStorage::<T>::put(self.initial_min_compliance);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Met à jour le score de conformité de l'appelant.
        ///
        /// Le nouveau score doit être supérieur ou égal au seuil minimal de conformité.
        #[pallet::weight(10_000)]
        pub fn update_compliance_status(
            origin: OriginFor<T>,
            new_score: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let min_level = Self::min_compliance_level();
            ensure!(new_score >= min_level, Error::<T>::ComplianceLevelTooLow);

            ComplianceStatus::<T>::insert(&who, new_score);
            let timestamp = T::TimeProvider::now().as_secs();
            ComplianceHistory::<T>::mutate(|logs| {
                logs.push(ComplianceLog {
                    account: who.clone(),
                    score: new_score,
                    timestamp,
                })
            });
            Self::deposit_event(Event::ComplianceUpdated(who, new_score));
            Ok(())
        }

        /// Met à jour le seuil minimal de conformité.
        /// Seul Root peut appeler cette fonction.
        #[pallet::weight(10_000)]
        pub fn update_min_compliance_level(
            origin: OriginFor<T>,
            new_min_
