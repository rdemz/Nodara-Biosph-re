#[cfg(test)]
mod tests {
    use super::*;
    use crate::pallet::Config;
    use frame_support::{assert_ok, assert_err};
    use sp_runtime::traits::SaturatedConversion;
    use sp_std::vec;

    // Mock configuration for nodara_marketplace
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
        type MaxAssetMetadataLength = sp_runtime::traits::ConstU32<256>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<TestConfig>().unwrap();
        storage.into()
    }

    #[test]
    fn test_register_asset() {
        new_test_ext().execute_with(|| {
            let metadata = b"Asset Metadata".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::register_asset(RawOrigin::Signed(1).into(), 100, metadata.clone()));
            let asset = <pallet::Assets<TestConfig>>::get(&100).unwrap();
            assert_eq!(asset.metadata, metadata);
            assert_eq!(asset.owner, 1);
        });
    }

    #[test]
    fn test_place_and_cancel_order() {
        new_test_ext().execute_with(|| {
            // Create a sample order for a buy order
            let order = pallet::Order {
                id: 1,
                asset_id: 100,
                order_type: pallet::OrderType::Buy,
                price: 50,
                quantity: 10,
                account: 1,
                timestamp: 1000,
            };
            // Place the order
            assert_ok!(<pallet::Pallet<TestConfig>>::place_order(RawOrigin::Signed(1).into(), order.clone()));
            // Cancel the order
            assert_ok!(<pallet::Pallet<TestConfig>>::cancel_order(RawOrigin::Signed(1).into(), 1, pallet::OrderType::Buy));
        });
    }

    #[test]
    fn test_execute_trade() {
        new_test_ext().execute_with(|| {
            // Register asset first
            let metadata = b"Asset Metadata".to_vec();
            assert_ok!(<pallet::Pallet<TestConfig>>::register_asset(RawOrigin::Signed(1).into(), 100, metadata));
            // Place buy order and sell order
            let buy_order = pallet::Order {
                id: 1,
                asset_id: 100,
                order_type: pallet::OrderType::Buy,
                price: 50,
                quantity: 10,
                account: 1,
                timestamp: 1000,
            };
            let sell_order = pallet::Order {
                id: 2,
                asset_id: 100,
                order_type: pallet::OrderType::Sell,
                price: 50,
                quantity: 10,
                account: 2,
                timestamp: 1000,
            };
            assert_ok!(<pallet::Pallet<TestConfig>>::place_order(RawOrigin::Signed(1).into(), buy_order));
            assert_ok!(<pallet::Pallet<TestConfig>>::place_order(RawOrigin::Signed(2).into(), sell_order));
            // Execute trade between buy order and sell order
            let trade = pallet::Trade {
                id: 1,
                buy_order_id: 1,
                sell_order_id: 2,
                asset_id: 100,
                price: 50,
                quantity: 10,
                timestamp: 2000,
            };
            assert_ok!(<pallet::Pallet<TestConfig>>::execute_trade(RawOrigin::Signed(1).into(), trade));
        });
    }
}
