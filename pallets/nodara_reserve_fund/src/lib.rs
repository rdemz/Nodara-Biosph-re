#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

/// # Nodara Reserve Fund Module - Advanced & Improved Version
///
/// Ce module gère le fonds de réserve du réseau Nodara, en intégrant
/// toutes les améliorations identifiées par rapport à la version de base :
/// - **Horodatage Dynamique :** Utilisation de `pallet_timestamp` pour obtenir des timestamps fiables.
/// - **Gestion Optimisée des Fonds :** Contributions, retraits avec vérification de seuil minimal.
/// - **Redistribution Automatique :** Redistribution de l'excédent de fonds via un hook périodique.
/// - **DAO Gouvernance :** Extrinsèque réservée à une origine DAO pour mettre à jour les paramètres critiques.
/// - **Audit Logging :** Enregistrement détaillé de chaque opération pour une traçabilité complète.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Get, EnsureOrigin},
    };
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;

    /// Structure d'un enregistrement d'opération sur le fonds de réserve.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct ReserveRecord {
        /// Timestamp de l'opération, obtenu via le pallet_timestamp.
        pub timestamp: u64,
        /// Solde précédent avant l'opération.
        pub previous_balance: u128,
        /// Nouveau solde après l'opération.
        pub new_balance: u128,
        /// Description ou raison de l'opération.
        pub operation: Vec<u8>,
    }

    /// État global du fonds de réserve.
    ///
    /// On conserve le solde actuel ainsi qu'un historique détaillé des opérations.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct ReserveFundState {
        /// Solde actuel du fonds de réserve.
        pub balance: u128,
        /// Historique des opérations sur le fonds.
        pub history: Vec<ReserveRecord>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Type d'événement utilisé par le runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Solde initial (baseline) du fonds de réserve lors de l'initialisation.
        #[pallet::constant]
        type BaselineReserve: Get<u128>;
        /// Origine autorisée à mettre à jour les paramètres critiques via DAO.
        type DaoOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// Seuil minimal de solde à maintenir pour autoriser un retrait (en pourcentage du baseline).
        #[pallet::constant]
        type MinimumReserveRatio: Get<u8>;
    }

    /// Stockage de l'état du fonds de réserve.
    #[pallet::storage]
    #[pallet::getter(fn reserve_state)]
    pub type ReserveFundStorage<T: Config> = StorageValue<_, ReserveFundState, ValueQuery>;

    /// Paramètre de gouvernance : seuil de redistribution.
    /// Si le solde dépasse ce seuil, l'excédent est redistribué automatiquement.
    #[pallet::storage]
    #[pallet::getter(fn redistribution_threshold)]
    pub type RedistributionThreshold<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mise à jour du fonds de réserve : (solde précédent, nouveau solde, opération).
        ReserveUpdated(u128, u128, Vec<u8>),
        /// Seuil de redistribution mis à jour par l'origine DAO.
        RedistributionThresholdUpdated(u128),
        /// Redistribution automatique effectuée (montant redistribué).
        FundsRedistributed(u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Opération invalide, par exemple retrait supérieur au solde disponible.
        InvalidOperation,
        /// Retrait non autorisé car le solde resterait en dessous du seuil minimal requis.
        InsufficientReserve,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Hooks permettant l'automatisation (ici, redistribution automatique en fin de bloc).
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            if let Some(amount) = Self::redistribute_funds() {
                Self::deposit_event(Event::FundsRedistributed(amount));
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise le fonds de réserve avec le solde de base.
        /// Seul l'appelant d'origine `Root` peut exécuter cette extrinsèque.
        #[pallet::weight(10_000)]
        pub fn initialize_reserve(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let now = <timestamp::Pallet<T>>::get();
            let baseline = T::BaselineReserve::get();
            let state = ReserveFundState {
                balance: baseline,
                history: vec![ReserveRecord {
                    timestamp: now,
                    previous_balance: 0,
                    new_balance: baseline,
                    operation: b"Initialization".to_vec(),
                }],
            };
            <ReserveFundStorage<T>>::put(state);
            // Par défaut, on fixe le seuil de redistribution à 150% du baseline.
            RedistributionThreshold::<T>::put(baseline.saturating_mul(150u128) / 100);
            Ok(())
        }

        /// Ajoute une contribution au fonds de réserve.
        ///
        /// La contribution est ajoutée au solde actuel et l'opération est enregistrée dans l'historique.
        #[pallet::weight(10_000)]
        pub fn contribute(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_add(amount);
            let now = <timestamp::Pallet<T>>::get();
            state.history.push(ReserveRecord {
                timestamp: now,
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
        /// Le retrait est autorisé uniquement si, après opération, le solde reste au-dessus du seuil minimal
        /// (défini en pourcentage du baseline).
        #[pallet::weight(10_000)]
        pub fn withdraw(origin: OriginFor<T>, amount: u128, description: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let mut state = <ReserveFundStorage<T>>::get();
            // Calcul du seuil minimal requis.
            let min_required = T::BaselineReserve::get()
                .saturating_mul(T::MinimumReserveRatio::get() as u128)
                / 100;
            ensure!(state.balance >= amount, Error::<T>::InvalidOperation);
            ensure!(state.balance.saturating_sub(amount) >= min_required, Error::<T>::InsufficientReserve);
            let previous_balance = state.balance;
            state.balance = state.balance.saturating_sub(amount);
            let now = <timestamp::Pallet<T>>::get();
            state.history.push(ReserveRecord {
                timestamp: now,
                previous_balance,
                new_balance: state.balance,
                operation: description.clone(),
            });
            <ReserveFundStorage<T>>::put(state);
            Self::deposit_event(Event::ReserveUpdated(previous_balance, <ReserveFundStorage<T>>::get().balance, description));
            Ok(())
        }

        /// Permet à une origine DAO de mettre à jour le seuil de redistribution.
        ///
        /// Cette extrinsèque permet de modifier dynamiquement le seuil au-delà duquel l'excédent sera redistribué.
        #[pallet::weight(10_000)]
        pub fn update_redistribution_threshold(origin: OriginFor<T>, new_threshold: u128) -> DispatchResult {
            T::DaoOrigin::ensure_origin(origin)?;
            RedistributionThreshold::<T>::put(new_threshold);
            Self::deposit_event(Event::RedistributionThresholdUpdated(new_threshold));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Redistribution automatique des fonds.
        ///
        /// Si le solde dépasse le seuil défini, l'excédent est extrait et redistribué.
        /// Cette fonction retourne `Some(montant)` si une redistribution a été effectuée,
        /// ou `None` sinon.
        fn redistribute_funds() -> Option<u128> {
            let mut state = <ReserveFundStorage<T>>::get();
            let threshold = RedistributionThreshold::<T>::get();
            if state.balance > threshold {
                let excess = state.balance.saturating_sub(threshold);
                let previous_balance = state.balance;
                state.balance = threshold;
                let now = <timestamp::Pallet<T>>::get();
                state.history.push(ReserveRecord {
                    timestamp: now,
                    previous_balance,
                    new_balance: state.balance,
                    operation: b"Automatic redistribution".to_vec(),
                });
                <ReserveFundStorage<T>>::put(state);
                return Some(excess);
            }
            None
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_redistribution_threshold: u128,
        pub _marker: sp_std::marker::PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                // Par défaut, seuil de redistribution à 150% du baseline.
                initial_redistribution_threshold: T::BaselineReserve::get().saturating_mul(150u128) / 100,
                _marker: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            RedistributionThreshold::<T>::put(self.initial_redistribution_threshold);
        }
    }

    // Des tests complets sont également inclus pour vérifier le bon fonctionnement des opérations.
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
                Timestamp: timestamp::Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineReserve: u128 = 1_000_000;
            pub const MinimumReserveRatio: u8 = 50; // 50% du baseline
            pub const MinimumPeriod: u64 = 1;
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

        impl timestamp::Config for Test {
            type Moment = u64;
            type OnTimestampSet = ();
            type MinimumPeriod = MinimumPeriod;
            type WeightInfo = ();
        }

        impl Config for Test {
            type RuntimeEvent = ();
            type BaselineReserve = BaselineReserve;
            type DaoOrigin = frame_system::EnsureRoot<u64>;
            type MinimumReserveRatio = MinimumReserveRatio;
        }

        #[test]
        fn initialize_and_contribute_work() {
            // Initialisation par Root.
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get());
            assert_eq!(state.history.len(), 1);
            // Contribution.
            let account = 1;
            let contribution = 500_000;
            let desc = b"Test contribution".to_vec();
            assert_ok!(ReserveFundModule::contribute(system::RawOrigin::Signed(account).into(), contribution, desc));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get() + contribution);
            assert_eq!(state.history.len(), 2);
        }

        #[test]
        fn withdraw_validates_balance() {
            let account = 1;
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            let contribution = 500_000;
            assert_ok!(ReserveFundModule::contribute(system::RawOrigin::Signed(account).into(), contribution, b"Contribution".to_vec()));
            // Retrait autorisé.
            let withdraw_amount = 300_000;
            assert_ok!(ReserveFundModule::withdraw(system::RawOrigin::Signed(account).into(), withdraw_amount, b"Withdrawal".to_vec()));
            let state = ReserveFundModule::reserve_state();
            assert_eq!(state.balance, BaselineReserve::get() + contribution - withdraw_amount);
        }

        #[test]
        fn withdraw_fails_for_insufficient_reserve() {
            let account = 1;
            assert_ok!(ReserveFundModule::initialize_reserve(system::RawOrigin::Root.into()));
            // Retrait qui mettrait le solde en dessous du seuil minimal.
            assert_err!(
                ReserveFundModule::withdraw(system::RawOrigin::Signed(account).into(), BaselineReserve::get(), b"Test".to_vec()),
                Error::<Test>::InsufficientReserve
            );
        }
    }
}
