#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_standards
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
        type MaxStandardLength = sp_runtime::traits::ConstU32<256>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_define_standard() {
        new_test_ext().execute_with(|| {
            let id = b"Standard1".to_vec();
            let description = b"Benchmark Description".to_vec();
            let parameters = b"Benchmark Parameters".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::define_standard(RawOrigin::Signed(1).into(), id.clone(), description.clone(), parameters.clone()));
            let standard = <pallet::Standards<TestConfig>>::get(&id).unwrap();
            assert_eq!(standard.description, description);
            assert_eq!(standard.parameters, parameters);
        });
    }

    #[test]
    fn test_update_standard() {
        new_test_ext().execute_with(|| {
            let id = b"Standard1".to_vec();
            let description = b"Initial Description".to_vec();
            let parameters = b"Initial Parameters".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::define_standard(RawOrigin::Signed(1).into(), id.clone(), description, parameters));
            let new_description = b"New Description".to_vec();
            let new_parameters = b"New Parameters".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::update_standard(RawOrigin::Signed(1).into(), id.clone(), new_description.clone(), new_parameters.clone()));
            let standard = <pallet::Standards<TestConfig>>::get(&id).unwrap();
            assert_eq!(standard.description, new_description);
            assert_eq!(standard.parameters, new_parameters);
        });
    }

    #[test]
    fn test_verify_compliance() {
        new_test_ext().execute_with(|| {
            let id = b"Standard1".to_vec();
            let description = b"Test Description".to_vec();
            let parameters = b"TestParams".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::define_standard(RawOrigin::Signed(1).into(), id.clone(), description, parameters.clone()));
            let operation_data = b"This operation includes TestParams within the data".to_vec();
            let result = <pallet::Pallet<TestConfig>>::verify_compliance(id.clone(), operation_data);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        });
    }
}
