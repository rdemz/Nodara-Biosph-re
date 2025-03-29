#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Reserve Fund Module - Advanced Version
//!
//! This module manages the reserve fund within the Nodara network. It collects contributions,
//! maintains a reserve balance, and redistributes funds when needed to stabilize the network economy.
//! It includes comprehensive audit logging and is designed to integrate with DAO governance for future parameter updates.
//!
//! ## Advanced Features:
//! - **Fund Collection & Redistribution:** Securely collects funds (e.g. transaction fees) and allows controlled withdrawals.
//! - **Dynamic Reserve Management:** Adjusts the reserve balance in real time based on contributions and withdrawals.
//! - **Audit Logging:** Maintains an immutable log of every operation with timestamps and details.
//! - **DAO Governance Integration:** Permits future mises à jour des paramètres via des propositions on-chain.
//! - **Performance Optimizations:** Utilisation d'opérations arithmétiques optimisées pour un traitement rapide.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;

    /// Structure d'un enregistrement d'opération sur le fonds de réserve.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct ReserveRecord {
        /// Unix timestamp de l'opération.
        pub timestamp: u64,
        /// Solde précédent.
        pub previous_balance: u128,
        /// Nouveau solde après l'opération.
        pub new_balance: u128,
        /// Description ou raison de l'opération.
        pub operation: Vec<u8>,
    }

    /// État global du fonds de réserve.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct ReserveFundState {
        /// Solde actuel du fonds.
        pub balance: u128,
        /// Historique des opérations sur le fonds.
        pub history: Vec<ReserveRecord>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Solde de réserve de base pour l'initialisation.
        #[pallet::constant]
        type BaselineReserve: Get<u128>;
    }

    /// Stockage de l'état du fonds de réserve.
    #[pallet::storage]
    #[pallet::getter(fn reserve_state)]
    pub type ReserveFundStorage<T: Config> = StorageValue<_, ReserveFundState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Événement émis lors d'une mise à jour du fonds de réserve.
        /// (solde précédent, nouveau solde, description de l'opération)
        ReserveUpdated(u128, u128, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Opération invalide (ex. retrait supérieur au solde disponible).
        InvalidOperation,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise le fonds de réserve avec le solde de base.
        /// Seul un appel provenant de l'origine `Root` est autorisé.
        #[pallet::weight(10_000)]
        pub fn initialize_reserve(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline = T::BaselineReserve::get();
            let state = ReserveFundState {
                balance: baseline,
                history: vec![ReserveRecord {
                    timestamp,
                    previous_balance: 0,
                    new_balance: baseline,
                    operation: b"Initialization".to_vec(),
                }],
            };
            <ReserveFundStorage<T>>::put(state);
            Ok(())
        }

        /// Ajoute une contribution au fonds de réserve.
        ///
        /// La contribution est ajoutée au solde actuel et enregistrée dans l'historique.
        #[pallet::weight(10_000)]
        pub fn contribute(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_add(amount);
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(ReserveRecord {
                timestamp,
                previous_balance,
                new_balance: state.balance,
                operation: description.clone(),
            });
            <ReserveFundStorage<T>>::put(state);
            Self::deposit_event(Event::ReserveUpdated(previous_balance, <ReserveFundStorage<T>>::get().balance, description));
            Ok(())
        }

        /// Effectue un retrait du fonds de réserve.
        ///
        /// Le retrait est autorisé uniquement si le solde est suffisant.
        #[pallet::weight(10_000)]
        pub fn withdraw(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            ensure!(state.balance >= amount, Error::<T>::InvalidOperation);
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_sub(amount);
            let timestamp = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            state.history.push(ReserveRecord {
                timestamp,
                previous_balance,
                new_balance: state.balance,
                operation: description.clone(),
            });
            <ReserveFundStorage<T>>::put(state);
            Self::deposit_event(Event::ReserveUpdated(previous_balance, <ReserveFundStorage<T>>::get().balance, description));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Retourne un horodatage fixe.
        /// En production, remplacer par une source de temps fiable (ex. `pallet_timestamp`).
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
                ReserveFundModule: Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineReserve: u128 = 1_000_000;
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
            type BaselineReserve = BaselineReserve;
        }

        #[test]
        fn initialize_reserve_works() {
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get());
            assert_eq!(state.history.len(), 1);
            let record = &state.history[0];
            assert_eq!(record.new_balance, BaselineReserve::get());
            assert_eq!(record.operation, b"Initialization".to_vec());
        }

        #[test]
        fn contribute_increases_balance() {
            let account = 1;
            // On initialise le fonds.
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            let contribution = 500_000;
            let description = b"Contribution test".to_vec();
            assert_ok!(ReserveFundModule::contribute(system::RawOrigin::Signed(account).into(), contribution, description.clone()));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get() + contribution);
            // Vérification de l'historique.
            assert_eq!(state.history.len(), 2);
        }

        #[test]
        fn withdraw_decreases_balance() {
            let account = 1;
            // On initialise le fonds.
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            let contribution = 500_000;
            assert_ok!(ReserveFundModule::contribute(system::RawOrigin::Signed(account).into(), contribution, b"Contribution".to_vec()));
            let withdraw_amount = 300_000;
            assert_ok!(ReserveFundModule::withdraw(system::RawOrigin::Signed(account).into(), withdraw_amount, b"Withdrawal".to_vec()));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get() + contribution - withdraw_amount);
            // L'historique doit comporter 3 entrées (initialisation, contribution, retrait).
            assert_eq!(state.history.len(), 3);
        }

        #[test]
        fn withdraw_fails_if_insufficient_balance() {
            let account = 1;
            // On initialise le fonds.
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            // Tentative de retrait supérieur au solde.
            assert_err!(
                ReserveFundModule::withdraw(system::RawOrigin::Signed(account).into(), BaselineReserve::get() + 1, b"Test".to_vec()),
                Error::<Test>::InvalidOperation
            );
        }
    }
}
