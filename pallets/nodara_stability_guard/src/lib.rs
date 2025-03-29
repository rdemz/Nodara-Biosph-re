#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

/// # Nodara Stability Guard Module - Extreme Version (Final)
///
/// Ce module surveille la volatilité du réseau et ajuste dynamiquement un paramètre de stabilité pour préserver la santé du réseau.
/// Il utilise une moyenne mobile exponentielle (EMA) avec un mécanisme de dampening pour lisser les fluctuations de volatilité.
/// Le nouveau paramètre est contraint entre des bornes minimales et maximales, et chaque ajustement est historisé pour une auditabilité complète.
/// Le module intègre également une extrinsèque DAO permettant de mettre à jour dynamiquement la configuration.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::vec::Vec;
    use sp_runtime::RuntimeDebug;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;
    use sp_runtime::traits::SaturatedConversion;

    /// Structure représentant un enregistrement d'ajustement de stabilité.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct StabilityRecord {
        pub timestamp: u64,
        pub old_parameter: u32,
        pub new_parameter: u32,
        pub volatility: u32,
        pub new_ema: u32,
    }

    /// État global du module de stabilité.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
    pub struct StabilityState {
        pub current_parameter: u32,
        /// Moyenne mobile exponentielle de la volatilité.
        pub volatility_ema: u32,
        pub history: Vec<StabilityRecord>,
    }

    /// Configuration dynamique du module, modifiable par DAO.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct StabilityConfig {
        pub smoothing_factor: u32, // en pourcentage (ex: 30 pour 30%)
        pub dampening_factor: u32, // facteur pour atténuer l'ajustement (>= 1)
        pub min_parameter: u32,
        pub max_parameter: u32,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuration du module.
    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Type d'événement.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Paramètre de stabilité initial (valeur de base).
        #[pallet::constant]
        type BaselineParameter: Get<u32>;
        /// Valeur par défaut du facteur de lissage (en pourcentage).
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
        /// Valeur par défaut du facteur de dampening.
        #[pallet::constant]
        type DampeningFactor: Get<u32>;
        /// Valeur maximale autorisée pour le paramètre de stabilité.
        #[pallet::constant]
        type MaxStabilityParameter: Get<u32>;
        /// Valeur minimale autorisée pour le paramètre de stabilité.
        #[pallet::constant]
        type MinStabilityParameter: Get<u32>;
        /// Origine autorisée à mettre à jour la configuration DAO.
        type DaoOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// Stockage de l'état global du module.
    #[pallet::storage]
    #[pallet::getter(fn stability_state)]
    pub type StabilityStorage<T: Config> = StorageValue<_, StabilityState, ValueQuery>;

    /// Stockage de la configuration dynamique du module.
    #[pallet::storage]
    #[pallet::getter(fn stability_config)]
    pub type StabilityConfigStorage<T: Config> = StorageValue<_, StabilityConfig, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Émission lors d'un ajustement de stabilité : (ancien paramètre, nouveau paramètre, volatilité, nouvelle EMA)
        StabilityAdjusted(u32, u32, u32, u32),
        /// Configuration DAO mise à jour : (smoothing_factor, dampening_factor, min_parameter, max_parameter)
        ConfigurationUpdated(u32, u32, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Erreur lors de l'ajustement (par exemple, calcul erroné ou dépassement de bornes).
        AdjustmentError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise l'état du module avec la valeur de base et la configuration par défaut.
        /// Cette extrinsèque est réservée à une origine Root.
        #[pallet::weight(10_000)]
        pub fn initialize_stability(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            let now = <timestamp::Pallet<T>>::get();
            let baseline = T::BaselineParameter::get();
            let state = StabilityState {
                current_parameter: baseline,
                volatility_ema: 0,
                history: Vec::new(),
            };
            <StabilityStorage<T>>::put(state);
            // Initialisation de la configuration DAO à partir des constantes.
            let config = StabilityConfig {
                smoothing_factor: T::SmoothingFactor::get(),
                dampening_factor: T::DampeningFactor::get(),
                min_parameter: T::MinStabilityParameter::get(),
                max_parameter: T::MaxStabilityParameter::get(),
            };
            <StabilityConfigStorage<T>>::put(config);
            Self::deposit_event(Event::StabilityAdjusted(baseline, baseline, 0, 0));
            Ok(())
        }

        /// Met à jour la volatilité observée et ajuste le paramètre de stabilité.
        ///
        /// `volatility` représente la nouvelle mesure de volatilité.
        #[pallet::weight(10_000)]
        pub fn update_volatility(origin: OriginFor<T>, volatility: u32) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // Récupérer l'état et la configuration courants.
            let mut state = <StabilityStorage<T>>::get();
            let config = <StabilityConfigStorage<T>>::get();
            let now = <timestamp::Pallet<T>>::get();

            // Calcul de la nouvelle EMA :
            // EMA_new = (smoothing_factor * volatility + (100 - smoothing_factor) * EMA_prev) / 100.
            let new_ema = ((config.smoothing_factor.saturating_mul(volatility))
                + ((100u32.saturating_sub(config.smoothing_factor)).saturating_mul(state.volatility_ema)))
                / 100;

            // Calcul du delta de l'EMA.
            let ema_delta = new_ema as i32 - state.volatility_ema as i32;
            // Application du dampening pour atténuer l'ajustement.
            let delta = ema_delta / config.dampening_factor as i32;
            let mut new_parameter = (state.current_parameter as i32).saturating_add(delta) as u32;

            // Contrainte du nouveau paramètre aux bornes minimales et maximales.
            if new_parameter > config.max_parameter {
                new_parameter = config.max_parameter;
            } else if new_parameter < config.min_parameter {
                new_parameter = config.min_parameter;
            }

            // Création du record d'ajustement.
            let record = StabilityRecord {
                timestamp: now,
                old_parameter: state.current_parameter,
                new_parameter,
                volatility,
                new_ema,
            };

            // Mise à jour de l'état.
            state.current_parameter = new_parameter;
            state.volatility_ema = new_ema;
            state.history.push(record);

            <StabilityStorage<T>>::put(state);
            Self::deposit_event(Event::StabilityAdjusted(state.current_parameter, new_parameter, volatility, new_ema));
            Ok(())
        }

        /// Permet à une origine DAO de mettre à jour la configuration du module.
        ///
        /// Les paramètres mis à jour sont le facteur de lissage, le facteur de dampening,
        /// la borne minimale et la borne maximale pour le paramètre de stabilité.
        #[pallet::weight(10_000)]
        pub fn update_configuration(
            origin: OriginFor<T>,
            new_smoothing: u32,
            new_dampening: u32,
            new_min: u32,
            new_max: u32,
        ) -> DispatchResult {
            T::DaoOrigin::ensure_origin(origin)?;
            let config = StabilityConfig {
                smoothing_factor: new_smoothing,
                dampening_factor: new_dampening,
                min_parameter: new_min,
                max_parameter: new_max,
            };
            <StabilityConfigStorage<T>>::put(config.clone());
            Self::deposit_event(Event::ConfigurationUpdated(new_smoothing, new_dampening, new_min, new_max));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Des fonctions utilitaires supplémentaires peuvent être ajoutées ici si besoin.
    }

    #[cfg(feature = "std")]
    impl<T: Config> core::fmt::Debug for Pallet<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Nodara Stability Guard Module")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, parameter_types};
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
                StabilityGuardModule: Pallet,
                Timestamp: timestamp::Pallet,
            }
        );

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const BaselineParameter: u32 = 100;
            pub const SmoothingFactor: u32 = 30; // 30%
            pub const DampeningFactor: u32 = 2;   // Divise le delta par 2
            pub const MaxStabilityParameter: u32 = 200;
            pub const MinStabilityParameter: u32 = 50;
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
            type BaselineParameter = BaselineParameter;
            type SmoothingFactor = SmoothingFactor;
            type DampeningFactor = DampeningFactor;
            type MaxStabilityParameter = MaxStabilityParameter;
            type MinStabilityParameter = MinStabilityParameter;
            type DaoOrigin = frame_system::EnsureRoot<u64>;
        }

        #[test]
        fn initialize_stability_works() {
            assert_ok!(StabilityGuardModule::initialize_stability(system::RawOrigin::Root.into()));
            let state = StabilityGuardModule::stability_state();
            assert_eq!(state.current_parameter, BaselineParameter::get());
            assert_eq!(state.volatility_ema, 0);
            assert!(state.history.is_empty());
        }

        #[test]
        fn update_volatility_adjusts_parameter() {
            assert_ok!(StabilityGuardModule::initialize_stability(system::RawOrigin::Root.into()));
            // Première mise à jour avec volatilité = 80.
            assert_ok!(StabilityGuardModule::update_volatility(system::RawOrigin::Signed(1).into(), 80));
            let state = StabilityGuardModule::stability_state();
            // Calcul attendu de l'EMA: (30*80 + 70*0)/100 = 24.
            // Delta = (24 - 0) / 2 = 12. Nouveau paramètre = 100 + 12 = 112.
            assert_eq!(state.current_parameter, 112);
            assert_eq!(state.volatility_ema, 24);
            assert_eq!(state.history.len(), 1);
            // Deuxième mise à jour avec volatilité = 120.
            assert_ok!(StabilityGuardModule::update_volatility(system::RawOrigin::Signed(1).into(), 120));
            let state = StabilityGuardModule::stability_state();
            // Nouvelle EMA = (30*120 + 70*24)/100 = (3600 + 1680)/100 = 52.8 arrondi à 52.
            // Delta = (52 - 24) / 2 = 14. Nouveau paramètre = 112 + 14 = 126.
            assert_eq!(state.current_parameter, 126);
            assert_eq!(state.volatility_ema, 52);
            assert_eq!(state.history.len(), 2);
        }

        #[test]
        fn update_configuration_works() {
            assert_ok!(StabilityGuardModule::initialize_stability(system::RawOrigin::Root.into()));
            // Mise à jour de la configuration DAO.
            assert_ok!(StabilityGuardModule::update_configuration(
                system::RawOrigin::Root.into(),
                40,  // new smoothing_factor
                3,   // new dampening_factor
                60,  // new min_parameter
                180  // new max_parameter
            ));
            let config = StabilityGuardModule::stability_config();
            assert_eq!(config.smoothing_factor, 40);
            assert_eq!(config.dampening_factor, 3);
            assert_eq!(config.min_parameter, 60);
            assert_eq!(config.max_parameter, 180);
        }
    }
}

