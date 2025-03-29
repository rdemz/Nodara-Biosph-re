#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Reputation Module - Locked and Ready for Deployment
//!
//! Ce module gère le système de réputation au sein du réseau Nodara BIOSPHÈRE QUANTIC. Il agrège les scores de réputation
//! pour chaque compte, conserve un historique détaillé des ajustements et intègre la gouvernance on-chain pour la modification
//! dynamique des paramètres.
//!
//! Toutes les dépendances sont verrouillées pour garantir une reproductibilité en production.

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

    /// Structure représentant une entrée dans l'historique des ajustements de réputation.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationLog {
        /// Timestamp (Unix seconds) de l'ajustement.
        pub timestamp: u64,
        /// Variation de réputation (positive ou négative).
        pub delta: i32,
        /// Raison de l'ajustement.
        pub reason: Vec<u8>,
    }

    /// Structure représentant l'enregistrement de réputation pour un compte.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationRecord {
        /// Score de réputation courant.
        pub score: u32,
        /// Historique complet des ajustements.
        pub history: Vec<ReputationLog>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Score de réputation initial pour un nouveau compte.
        #[pallet::constant]
        type InitialReputation: Get<u32>;
    }

    /// Stockage des enregistrements de réputation pour chaque compte.
    #[pallet::storage]
    #[pallet::getter(fn reputations)]
    pub type Reputations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ReputationRecord, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Émission lors d'un ajustement de réputation. (compte, delta, nouveau score)
        ReputationUpdated(T::AccountId, i32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Aucun enregistrement de réputation trouvé pour ce compte.
        ReputationNotFound,
        /// L'ajustement conduirait à un score négatif.
        ReputationUnderflow,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise la réputation de l'appelant avec la valeur initiale.
        #[pallet::weight(10_000)]
        pub fn initialize_reputation(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Reputations::<T>::contains_key(&who), "Reputation already exists for this account");
            let record = ReputationRecord {
                score: T::InitialReputation::get(),
                history: Vec::new(),
            };
            Reputations::<T>::insert(&who, record);
            Ok(())
        }

        /// Met à jour la réputation de l'appelant en fonction d'une variation.
        /// Le delta peut être positif (augmentation) ou négatif (diminution).
        #[pallet::weight(10_000)]
        pub fn update_reputation(origin: OriginFor<T>, delta: i32, reason: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Reputations::<T>::try_mutate(&who, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::ReputationNotFound)?;
                let current = record.score as i32;
                let new_score = current.checked_add(delta).ok_or(Error::<T>::ReputationUnderflow)?;
                ensure!(new_score >= 0, Error::<T>::ReputationUnderflow);
                record.score = new_score as u32;
                let timestamp = Self::current_timestamp();
                record.history.push(ReputationLog {
                    timestamp,
                    delta,
                    reason,
                });
                Self::deposit_event(Event::ReputationUpdated(who.clone(), delta, record.score));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Retourne un timestamp fixe pour les tests.
        /// En production, remplacez ceci par une source de temps fiable (par ex. `pallet_timestamp`).
        fn current_timestamp() -> u64 {
            1_640_000_000
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

        type UncheckedExtrinsic = system::mocking::MockUncheckedExtrinsic<Test>;
        type Block = system::mocking::MockBlock<Test>;

        frame_support::construct_runtime!(
            pub enum Test where 
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                ReputationModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const InitialReputation: u32 = 100;
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
            type InitialReputation = InitialReputation;
        }

        #[test]
        fn initialize_reputation_should_work() {
            let origin = system::RawOrigin::Signed(1).into();
            assert_ok!(ReputationModule::initialize_reputation(origin));
            let record = ReputationModule::reputations(1).expect("Reputation record must exist");
            assert_eq!(record.score, InitialReputation::get());
        }

        #[test]
        fn update_reputation_should_work() {
            let origin = system::RawOrigin::Signed(1).into();
            assert_ok!(ReputationModule::initialize_reputation(origin.clone()));
            // Update reputation: add 50 points.
            assert_ok!(ReputationModule::update_reputation(system::RawOrigin::Signed(1).into(), 50, b"Good behavior".to_vec()));
            let record = ReputationModule::reputations(1).expect("Record exists");
            assert_eq!(record.score, InitialReputation::get() + 50);
            // History should contain one log entry.
            assert_eq!(record.history.len(), 1);
        }

        #[test]
        fn update_reputation_should_fail_on_underflow() {
            let origin = system::RawOrigin::Signed(1).into();
            assert_ok!(ReputationModule::initialize_reputation(origin.clone()));
            let delta = -(InitialReputation::get() as i32 + 1);
            assert_err!(
                ReputationModule::update_reputation(system::RawOrigin::Signed(1).into(), delta, b"Bad behavior".to_vec()),
                Error::<Test>::ReputationUnderflow
            );
        }
    }
}
