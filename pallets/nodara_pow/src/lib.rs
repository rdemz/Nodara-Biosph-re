#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_pow - Legendary Edition (Extreme Version)
//!
//! Ce module implémente un mécanisme Proof-of-Work (PoW) dynamique et biomimétique pour Nodara BIOSPHÈRE QUANTIC.
//! Il valide de manière sécurisée les soumissions de travail par les mineurs, ajuste dynamiquement la difficulté
//! en fonction des conditions du réseau et journalise toutes les opérations pour une auditabilité complète.
//!
//! Les fonctionnalités avancées incluent :
//! - Vérification de signature basée sur Blake2-128 (simulation).
//! - Contrôle strict des valeurs de work et signal.
//! - Historique complet des ajustements de difficulté.

use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::Get,
    transactional,
};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::{RuntimeDebug, traits::SaturatedConversion};
use parity_scale_codec::{Encode, Decode};

/// Structure représentant l'état de PoW.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct PowState {
    /// Difficulté actuelle du PoW.
    pub difficulty: u32,
    /// Total cumulé de travail soumis.
    pub total_work: u32,
    /// Historique des ajustements : (timestamp, ancien niveau, nouveau niveau, signal soumis).
    pub history: Vec<(u64, u32, u32, u32)>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_io::hashing::blake2_128;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Difficulté de minage de base.
        #[pallet::constant]
        type BaselineDifficulty: Get<u32>;
        /// Facteur de lissage pour l'ajustement de la difficulté (doit être > 0).
        #[pallet::constant]
        type PowSmoothingFactor: Get<u32>;
    }

    /// Stockage de l'état PoW.
    #[pallet::storage]
    #[pallet::getter(fn pow_state)]
    pub type PowStateStorage<T: Config> = StorageValue<_, PowState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Un travail a été soumis et validé. (mineur, work_value)
        PowSubmitted(T::AccountId, u32),
        /// La difficulté a été ajustée. (ancien niveau, nouveau niveau, signal)
        DifficultyAdjusted(u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// La soumission de travail est invalide (work_value <= 0).
        InvalidWork,
        /// Le travail soumis ne satisfait pas la difficulté requise.
        WorkRejected,
        /// La vérification de la signature a échoué.
        SignatureVerificationFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état PoW avec la difficulté de base.
        ///
        /// Doit être appelé par Root.
        #[pallet::weight(10_000)]
        pub fn initialize_pow(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineDifficulty::get();
            let state = PowState {
                difficulty: baseline,
                total_work: 0,
                history: vec![(now, 0, baseline, 0)],
            };
            <PowStateStorage<T>>::put(state);
            Ok(())
        }

        /// Soumet un travail de minage.
        ///
        /// Le travail est validé si work_value est >= difficulté actuelle.
        /// La signature doit correspondre au hash Blake2-128 du payload (simulation).
        #[pallet::weight(10_000)]
        pub fn submit_work(
            origin: OriginFor<T>,
            work_value: u32,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            ensure!(work_value > 0, Error::<T>::InvalidWork);
            // Vérification de la signature (simulation)
            ensure!(Self::verify_signature(work_value, &signature), Error::<T>::SignatureVerificationFailed);
            let state = <PowStateStorage<T>>::get();
            // Vérification que le travail soumis satisfait la difficulté.
            ensure!(work_value >= state.difficulty, Error::<T>::WorkRejected);

            // Mise à jour du total de travail.
            <PowStateStorage<T>>::mutate(|s| {
                s.total_work = s.total_work.saturating_add(work_value);
            });

            Self::deposit_event(Event::PowSubmitted(miner, work_value));
            Ok(())
        }

        /// Ajuste la difficulté en fonction d'un signal.
        ///
        /// Le nouveau niveau de difficulté est calculé par :
        ///     new_difficulty = current_difficulty + (signal / PowSmoothingFactor)
        #[pallet::weight(10_000)]
        pub fn adjust_difficulty(
            origin: OriginFor<T>,
            signal: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            // Vérifier que le signal est positif.
            ensure!(signal > 0, Error::<T>::InvalidWork);
            let smoothing = T::PowSmoothingFactor::get();
            ensure!(smoothing > 0, "Smoothing factor must be non-zero");

            <PowStateStorage<T>>::mutate(|s| {
                let previous = s.difficulty;
                let adjustment = signal / smoothing;
                let new_difficulty = previous.saturating_add(adjustment);
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                s.history.push((now, previous, new_difficulty, signal));
                s.difficulty = new_difficulty;
            });
            let state = <PowStateStorage<T>>::get();
            let last_record = state.history.last().unwrap();
            Self::deposit_event(Event::DifficultyAdjusted(last_record.1, state.difficulty, signal));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Vérifie la signature du travail.
        /// Ici, nous simulons la vérification en comparant la signature au hash Blake2-128 du work_value encodé.
        fn verify_signature(work_value: u32, signature: &Vec<u8>) -> bool {
            let encoded = work_value.encode();
            let hash = blake2_128(&encoded);
            signature.len() == 16 && signature == &hash.to_vec()
        }
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
            PowModule: pallet::{Pallet, Call, Storage, Event<T>},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const BaselineDifficulty: u32 = 100;
        pub const PowSmoothingFactor: u32 = 10;
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
        type BaselineDifficulty = BaselineDifficulty;
        type PowSmoothingFactor = PowSmoothingFactor;
    }

    #[test]
    fn test_initialize_pow() {
        let origin = system::RawOrigin::Root.into();
        assert_ok!(PowModule::initialize_pow(origin));
        let state = PowModule::pow_state();
        assert_eq!(state.difficulty, BaselineDifficulty::get());
        assert_eq!(state.history.len(), 1);
        let record = &state.history[0];
        assert_eq!(record.new_level, BaselineDifficulty::get());
    }

    #[test]
    fn test_submit_work_should_work() {
        // Initialize state.
        assert_ok!(PowModule::initialize_pow(system::RawOrigin::Root.into()));
        let origin = system::RawOrigin::Signed(1).into();
        let work_value = 150;
        // Génère une signature valide en utilisant le hash Blake2-128 du work_value encodé.
        let signature = work_value.encode();
        let signature = sp_io::hashing::blake2_128(&signature).to_vec();
        // Work_value 150 >= difficulty 100, donc accepté.
        assert_ok!(PowModule::submit_work(origin, work_value, signature));
        let state = PowModule::pow_state();
        assert_eq!(state.total_work, work_value);
    }

    #[test]
    fn test_submit_work_should_fail_if_work_too_low() {
        assert_ok!(PowModule::initialize_pow(system::RawOrigin::Root.into()));
        let origin = system::RawOrigin::Signed(1).into();
        let work_value = 50; // en dessous de la difficulté de 100
        let signature = work_value.encode();
        let signature = sp_io::hashing::blake2_128(&signature).to_vec();
        assert_err!(
            PowModule::submit_work(origin, work_value, signature),
            Error::<Test>::WorkRejected
        );
    }

    #[test]
    fn test_adjust_difficulty() {
        // Initialize state.
        assert_ok!(PowModule::initialize_pow(system::RawOrigin::Root.into()));
        let origin = system::RawOrigin::Signed(1).into();
        // Avec signal 50 et smoothing factor 10, adjustment = 50/10 = 5.
        let signal = 50;
        assert_ok!(PowModule::adjust_difficulty(origin, signal));
        let state = PowModule::pow_state();
        assert_eq!(state.difficulty, BaselineDifficulty::get() + 5);
        assert_eq!(state.history.len(), 2);
    }
}
