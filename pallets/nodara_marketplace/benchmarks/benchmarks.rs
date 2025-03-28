#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use frame_system::RawOrigin;
use sp_std::vec::Vec;
use core::ops::Div;

benchmarks! {
    register_asset {
        let metadata: Vec<u8> = b"Benchmark Asset Metadata".to_vec();
    }: {
        <pallet::Pallet<T>>::register_asset(RawOrigin::Signed(account("user", 0, 0)).into(), 100, metadata.clone())?;
    }
    verify {
        let asset = <pallet::Assets<T>>::get(&100).unwrap();
        assert_eq!(asset.metadata, metadata);
    }

    place_order {
        let order = pallet::Order {
            id: 1,
            asset_id: 100,
            order_type: pallet::OrderType::Buy,
            price: 50,
            quantity: 10,
            account: account("user", 0, 0),
            timestamp: 1000,
        };
    }: {
        <pallet::Pallet<T>>::place_order(RawOrigin::Signed(account("user", 0, 0)).into(), order.clone())?;
    }
    verify {
        let orders = <pallet::OrderBook<T>>::get(100);
        assert!(orders.contains(&1));
    }

    cancel_order {
        let order = pallet::Order {
            id: 1,
            asset_id: 100,
            order_type: pallet::OrderType::Buy,
            price: 50,
            quantity: 10,
            account: account("user", 0, 0),
            timestamp: 1000,
        };
        <pallet::Pallet<T>>::place_order(RawOrigin::Signed(account("user", 0, 0)).into(), order.clone())?;
    }: {
        <pallet::Pallet<T>>::cancel_order(RawOrigin::Signed(account("user", 0, 0)).into(), 1, pallet::OrderType::Buy)?;
    }
    verify {
        let orders = <pallet::OrderBook<T>>::get(100);
        assert!(!orders.contains(&1));
    }

    execute_trade {
        // Register asset
        let metadata: Vec<u8> = b"Asset Metadata".to_vec();
        <pallet::Pallet<T>>::register_asset(RawOrigin::Signed(account("user", 0, 0)).into(), 100, metadata)?;
        // Place buy and sell orders
        let buy_order = pallet::Order {
            id: 1,
            asset_id: 100,
            order_type: pallet::OrderType::Buy,
            price: 50,
            quantity: 10,
            account: account("user", 0, 0),
            timestamp: 1000,
        };
        let sell_order = pallet::Order {
            id: 2,
            asset_id: 100,
            order_type: pallet::OrderType::Sell,
            price: 50,
            quantity: 10,
            account: account("user", 1, 0),
            timestamp: 1000,
        };
        <pallet::Pallet<T>>::place_order(RawOrigin::Signed(account("user", 0, 0)).into(), buy_order)?;
        <pallet::Pallet<T>>::place_order(RawOrigin::Signed(account("user", 1, 0)).into(), sell_order)?;
        let trade = pallet::Trade {
            id: 1,
            buy_order_id: 1,
            sell_order_id: 2,
            asset_id: 100,
            price: 50,
            quantity: 10,
            timestamp: 2000,
        };
    }: {
        <pallet::Pallet<T>>::execute_trade(RawOrigin::Signed(account("user", 0, 0)).into(), trade)?;
    }
    verify {
        let history = <pallet::TradesHistory<T>>::get();
        assert!(!history.is_empty());
    }
}

impl_benchmark_test_suite!(pallet::Pallet, crate::mock::new_test_ext(), crate::Test);
