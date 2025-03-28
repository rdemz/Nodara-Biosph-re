#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_interop
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
        type MaxPayloadLength = sp_runtime::traits::ConstU32<256>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_send_message() {
        new_test_ext().execute_with(|| {
            let payload = b"Test Interop Message".to_vec();
            let signature = b"ValidSignature".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::send_message(1, payload.clone(), signature));
            let message = <pallet::OutgoingMessages<TestConfig>>::get(1).unwrap();
            assert_eq!(message.payload, payload);
        });
    }

    #[test]
    fn test_receive_message() {
        new_test_ext().execute_with(|| {
            let payload = b"Incoming Message".to_vec();
            let signature = b"ValidSignature".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::receive_message(2, payload.clone(), signature));
            let message = <pallet::IncomingMessages<TestConfig>>::get(2).unwrap();
            assert_eq!(message.payload, payload);
        });
    }

    #[test]
    fn test_update_config() {
        new_test_ext().execute_with(|| {
            let new_config = b"New Interop Config".to_vec();
            let details = b"Config details".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::update_config(new_config.clone(), details.clone()));
        });
    }
}
