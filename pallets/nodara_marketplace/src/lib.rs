#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_marketplace - Legendary Edition
//!
//! This module implements a decentralized marketplace for asset exchange on the Nodara BIOSPHÈRE QUANTIC platform.
//! It allows users to register assets, place and cancel orders, and execute trades with complete transparency and security.
//! The module uses efficient data structures and robust error handling to ensure high performance and reliability.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Enum to represent order types.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum OrderType {
    Buy,
    Sell,
}

/// Structure representing an asset.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct Asset {
    pub id: u64,
    pub metadata: Vec<u8>,
    pub owner: u64,
}

/// Structure representing an order.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct Order {
    pub id: u64,
    pub asset_id: u64,
    pub order_type: OrderType,
    pub price: u32,
    pub quantity: u32,
    pub account: u64,
    pub timestamp: u64,
}

/// Structure representing a trade.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct Trade {
    pub id: u64,
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub asset_id: u64,
    pub price: u32,
    pub quantity: u32,
    pub timestamp: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Maximum allowed length for asset metadata.
        #[pallet::constant]
        type MaxAssetMetadataLength: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn assets)]
    pub type Assets<T: Config> = StorageMap<_, Blake2_128Concat, u64, Asset, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn buy_orders)]
    pub type BuyOrders<T: Config> = StorageMap<_, Blake2_128Concat, u64, Order, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn sell_orders)]
    pub type SellOrders<T: Config> = StorageMap<_, Blake2_128Concat, u64, Order, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn order_book)]
    pub type OrderBook<T: Config> = StorageMap<_, Blake2_128Concat, u64, Vec<u64>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn trades_history)]
    pub type TradesHistory<T: Config> = StorageValue<_, Vec<Trade>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a new asset is registered (asset ID, owner).
        AssetRegistered(u64, u64),
        /// Emitted when an order is placed (order ID, order type, asset ID).
        OrderPlaced(u64, OrderType, u64),
        /// Emitted when an order is cancelled (order ID).
        OrderCancelled(u64),
        /// Emitted when a trade is executed (trade ID, asset ID, quantity, price).
        TradeExecuted(u64, u64, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The asset metadata length exceeds the allowed maximum.
        AssetMetadataTooLong,
        /// The asset is already registered.
        AssetAlreadyRegistered,
        /// The asset does not exist.
        AssetNotFound,
        /// Order not found.
        OrderNotFound,
        /// Insufficient order quantity.
        InsufficientOrderQuantity,
        /// Invalid order parameters.
        InvalidOrder,
    }

    impl<T: Config> Pallet<T> {
        /// Registers a new asset on the marketplace.
        pub fn register_asset(origin: T::Origin, asset_id: u64, metadata: Vec<u8>) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            ensure!((metadata.len() as u32) <= T::MaxAssetMetadataLength::get(), Error::<T>::AssetMetadataTooLong);
            ensure!(!Assets::<T>::contains_key(&asset_id), Error::<T>::AssetAlreadyRegistered);
            let asset = Asset {
                id: asset_id,
                metadata: metadata.clone(),
                owner,
            };
            <Assets<T>>::insert(asset_id, asset);
            Self::deposit_event(Event::AssetRegistered(asset_id, owner));
            Ok(())
        }

        /// Places a new order in the marketplace.
        pub fn place_order(origin: T::Origin, order: Order) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            match order.order_type {
                OrderType::Buy => <BuyOrders<T>>::insert(order.id, order.clone()),
                OrderType::Sell => <SellOrders<T>>::insert(order.id, order.clone()),
            }
            // Update the order book for the given asset.
            OrderBook::<T>::mutate(order.asset_id, |orders| orders.push(order.id));
            Self::deposit_event(Event::OrderPlaced(order.id, order.order_type, order.asset_id));
            Ok(())
        }

        /// Cancels an existing order.
        pub fn cancel_order(origin: T::Origin, order_id: u64, order_type: OrderType) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            match order_type {
                OrderType::Buy => ensure!(<BuyOrders<T>>::contains_key(&order_id), Error::<T>::OrderNotFound),
                OrderType::Sell => ensure!(<SellOrders<T>>::contains_key(&order_id), Error::<T>::OrderNotFound),
            }
            match order_type {
                OrderType::Buy => <BuyOrders<T>>::remove(order_id),
                OrderType::Sell => <SellOrders<T>>::remove(order_id),
            }
            Self::deposit_event(Event::OrderCancelled(order_id));
            Ok(())
        }

        /// Executes a trade by matching a buy order with a sell order.
        pub fn execute_trade(origin: T::Origin, trade: Trade) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!(<BuyOrders<T>>::contains_key(&trade.buy_order_id), Error::<T>::OrderNotFound);
            ensure!(<SellOrders<T>>::contains_key(&trade.sell_order_id), Error::<T>::OrderNotFound);
            // For simplicity, assume a direct match and update order book accordingly.
            // Remove matched orders.
            <BuyOrders<T>>::remove(trade.buy_order_id);
            <SellOrders<T>>::remove(trade.sell_order_id);
            // Log the trade in the trade history.
            TradesHistory::<T>::mutate(|history| history.push(trade.clone()));
            Self::deposit_event(Event::TradeExecuted(trade.id, trade.asset_id, trade.quantity, trade.price));
            Ok(())
        }
    }
}
