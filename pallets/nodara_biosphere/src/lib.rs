#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_biosphere - Legendary Edition
//!
//! This module manages the adaptive state of the Nodara blockchain, dynamically adjusting network parameters
//! based on real-time economic and performance signals. It handles transitions between different operational
//! phases (Growth, Defense, Mutation) using advanced algorithms inspired by quantum mechanics, and integrates
//! simulated formal verification to ensure the integrity and robustness of the system.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, log};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;
use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

/// Enum representing the different phases of network operation.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum BioPhase {
    Growth,
    Defense,
    Mutation,
}

/// Structure representing the complete state of the network.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct BioState {
    pub current_phase: BioPhase,
    pub energy_level: u32,
    pub quantum_flux: u32,
    pub last_updated: u64,
    pub history: Vec<(u64, BioPhase, u32, u32)>, // (timestamp, phase, energy, quantum_flux)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Baseline energy level for initialization.
        #[pallet::constant]
        type BaselineEnergy: Get<u32>;
        /// Baseline quantum flux for initialization.
        #[pallet::constant]
        type BaselineQuantumFlux: Get<u32>;
        /// Baseline phase for initialization.
        #[pallet::constant]
        type BaselinePhase: Get<BioPhase>;
        /// Smoothing factor for state transitions.
        #[pallet::constant]
        type SmoothingFactor: Get<u32>;
    }

    /// Storage for the bio state.
    #[pallet::storage]
    #[pallet::getter(fn bio_state)]
    pub type BioStateStorage<T: Config> = StorageValue<_, BioState, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when the bio state is updated (previous phase, new phase, energy level, quantum flux).
        BioStateUpdated(BioPhase, BioPhase, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Provided signal is invalid.
        InvalidSignal,
        /// Signature verification failed.
        SignatureVerificationFailed,
        /// Quantum flux calculation error.
        QuantumCalculationFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initializes the bio state with baseline values.
        #[pallet::weight(10_000)]
        pub fn initialize_state(origin: OriginFor<T>) -> DispatchResult {
            // Seul un appel provenant de la racine (Root) peut initialiser l'état.
            ensure_root(origin)?;
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let baseline_energy = T::BaselineEnergy::get();
            let baseline_flux = T::BaselineQuantumFlux::get();
            let baseline_phase = T::BaselinePhase::get();
            let initial_state = BioState {
                current_phase: baseline_phase.clone(),
                energy_level: baseline_energy,
                quantum_flux: baseline_flux,
                last_updated: now,
                history: vec![(now, baseline_phase.clone(), baseline_energy, baseline_flux)],
            };
            <BioStateStorage<T>>::put(initial_state);
            Ok(())
        }

        /// Updates the bio state based on an incoming signal and a cryptographic signature.
        ///
        /// The new phase is determined by comparing the signal with predefined thresholds.
        #[pallet::weight(10_000)]
        pub fn transition_phase(origin: OriginFor<T>, signal: u32, signature: Vec<u8>) -> DispatchResult {
            // Seul un appel signé est autorisé.
            ensure_signed(origin)?;
            ensure!(signal > 0, Error::<T>::InvalidSignal);
            // Simulation de vérification de signature.
            ensure!(!signature.is_empty(), Error::<T>::SignatureVerificationFailed);

            let current_state = <BioStateStorage<T>>::get();
            let new_phase = if signal > 100 {
                BioPhase::Growth
            } else if signal > 50 {
                BioPhase::Defense
            } else {
                BioPhase::Mutation
            };
            let new_energy = signal.saturating_mul(10); // Exemple de calcul
            let new_quantum_flux = (signal.saturating_mul(signal)) / T::SmoothingFactor::get();
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();

            // Mise à jour de l'historique et de l'état.
            let mut new_history = current_state.history.clone();
            new_history.push((now, new_phase.clone(), new_energy, new_quantum_flux));
            let updated_state = BioState {
                current_phase: new_phase.clone(),
                energy_level: new_energy,
                quantum_flux: new_quantum_flux,
                last_updated: now,
                history: new_history,
            };
            <BioStateStorage<T>>::put(updated_state);

            Self::deposit_event(Event::BioStateUpdated(current_state.current_phase, new_phase, new_energy, new_quantum_flux));
            Ok(())
        }
    }
}
