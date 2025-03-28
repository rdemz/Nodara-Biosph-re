#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use frame_support::sp_std::vec;

    // Mock configuration for testing purposes
    pub struct TestConfig;
    impl frame_system::Config for TestConfig {
        type BaseCallFilter = ();
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = ();
        type RuntimeCall = ();
        type BlockNumber = u64;
        type Hash = sp_core::H256;
        type Hashing = sp_core::H256;
        type AccountId = u64;
        type Lookup = ();
        type Header = ();
        type Index = u64;
        type BlockHashCount = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type RuntimeEvent = ();
        type Version = ();
        type PalletInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = ();
    }
    
    impl pallet::Config for TestConfig {
        type RuntimeEvent = ();
        type BaselineEnergy = sp_runtime::traits::ConstU32<50>;
        type BaselineQuantumFlux = sp_runtime::traits::ConstU32<10>;
        type BaselinePhase = sp_runtime::traits::ConstValue<BioPhase, BioPhase::Mutation>;
        type SmoothingFactor = sp_runtime::traits::ConstU32<10>;
    }
    
    // Dummy implementation for ConstValue trait for BioPhase
    pub struct BioPhaseValue;
    impl sp_runtime::traits::ConstValue<BioPhase> for BioPhaseValue {
        fn get() -> BioPhase {
            BioPhase::Mutation
        }
    }
    
    #[test]
    fn test_initialize_state() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<TestConfig>::initialize_state());
            let state = <pallet::BioStateStorage<TestConfig>>::get();
            assert_eq!(state.current_phase, BioPhase::Mutation);
            assert_eq!(state.energy_level, 50);
            assert_eq!(state.quantum_flux, 10);
        });
    }
    
    #[test]
    fn test_transition_phase_valid() {
        new_test_ext().execute_with(|| {
            // Initialize state first
            assert_ok!(Pallet::<TestConfig>::initialize_state());
            // Test transition with a valid signal
            let test_signal = 80;
            let dummy_signature = vec![1, 2, 3];
            assert_ok!(Pallet::<TestConfig>::transition_phase(test_signal, dummy_signature));
            let state = <pallet::BioStateStorage<TestConfig>>::get();
            // Depending on the threshold defined, assert that the state has been updated (example thresholds in lib.rs)
            // Ici, par exemple, signal > 50 mais < 100 donne Defense
            assert_eq!(state.current_phase, BioPhase::Defense);
        });
    }
}
