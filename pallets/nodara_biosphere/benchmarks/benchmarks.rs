#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

benchmarks! {
    initialize_state {
        // Setup before running initialize_state
    }: {
        // Call the initialize_state function
        Pallet::<T>::initialize_state()?;
    }
    verify {
        // Verify that the state has been initialized with the baseline values
        assert_eq!(Pallet::<T>::bio_state().energy_level, T::BaselineEnergy::get());
    }

    transition_phase {
        // Initialize first
        Pallet::<T>::initialize_state()?;
        let signal: u32 = 80;
        let signature: Vec<u8> = b"dummy_signature".to_vec();
    }: {
        Pallet::<T>::transition_phase(signal, signature)?;
    }
    verify {
        // Verify state update
        let expected_phase = if 80 > 100 { BioPhase::Growth } else if 80 > 50 { BioPhase::Defense } else { BioPhase::Mutation };
        assert_eq!(Pallet::<T>::bio_state().current_phase, expected_phase);
    }
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::Test);
