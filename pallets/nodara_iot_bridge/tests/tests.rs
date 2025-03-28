#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_iot_bridge
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
        type BaseTimeout = sp_runtime::traits::ConstU64<30>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_submit_iot_data() {
        new_test_ext().execute_with(|| {
            let payload = b"Sample IoT Data".to_vec();
            let device_id = b"Device123".to_vec();
            let signature = b"ValidSignature".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::submit_iot_data(1, payload.clone(), device_id.clone(), signature));
            let record = <pallet::IotData<TestConfig>>::get(1).unwrap();
            assert_eq!(record.payload, payload);
            assert_eq!(record.device_id, device_id);
        });
    }

    #[test]
    fn test_update_config() {
        new_test_ext().execute_with(|| {
            let new_config = b"NewConfigParams".to_vec();
            let details = b"Update for better performance".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::update_config(new_config.clone(), details.clone()));
        });
    }
}
