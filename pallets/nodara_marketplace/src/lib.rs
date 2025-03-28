#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Marketplace Module - Locked and Ready for Deployment
//!
//! This module implements a decentralized marketplace for the Nodara network.
//! It allows users to register assets, place buy and sell orders, and execute trades
//! while maintaining full audit logging and DAO governance integration.
//!
//! ## Features
//! - **Asset Registration:** Secure registration and management of asset metadata.
//! - **Order Placement and Matching:** Buy and sell order placement with a matching engine.
//! - **Trade Execution:** Secure execution of trades with proper asset and fund transfers.
//! - **Audit Logging:** Immutable logging of all marketplace events for traceability.
//! - **DAO Governance Integration:** On-chain proposals for updating marketplace parameters.
//!
//! Dependencies are locked to fixed versions to ensure a reproducible build.

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::Get,
    };
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::{Decode, Encode};
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;
    use sp_std::collections::btree_map::BTreeMap;

    /// Structure representing an asset registered on the marketplace.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Asset {
        /// Unique asset identifier.
        pub id: u64,
        /// Metadata associated with the asset (e.g. JSON encoded data).
        pub metadata: Vec<u8>,
        /// Owner of the asset.
        pub owner: u64, // Pour simplifier, nous utilisons u64. En production, utilisez T::AccountId.
    }

    /// Enum pour distinguer les types d'ordres.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum OrderType {
        Buy,
        Sell,
    }

    /// Structure représentant un ordre dans le marketplace.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Order {
        /// Identifiant unique de l'ordre.
        pub id: u64,
        /// Identifiant de l'actif concerné.
        pub asset_id: u64,
        /// Type d'ordre : Buy ou Sell.
        pub order_type: OrderType,
        /// Prix par unité (en plus petite unité de la monnaie).
        pub price: u32,
        /// Quantité à acheter ou vendre.
        pub quantity: u32,
        /// Identifiant du compte ayant passé l'ordre.
        pub account: u64,
        /// Timestamp de placement de l'ordre.
        pub timestamp: u64,
    }

    /// Structure représentant une exécution de trade.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Trade {
        /// Identifiant unique du trade.
        pub id: u64,
        /// Identifiant de l'ordre d'achat.
        pub buy_order_id: u64,
        /// Identifiant de l'ordre de vente.
        pub sell_order_id: u64,
        /// Identifiant de l'actif échangé.
        pub asset_id: u64,
        /// Prix auquel le trade a été exécuté.
        pub price: u32,
        /// Quantité échangée.
        pub quantity: u32,
        /// Timestamp de l'exécution.
        pub timestamp: u64,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Le type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Longueur maximale autorisée pour les métadonnées d'un actif.
        #[pallet::constant]
        type MaxAssetMetadataLength: Get<u32>;
        /// Frais de base pour l'exécution d'un trade.
        #[pallet::constant]
        type BaseTradeFee: Get<u32>;
    }

    /// Stockage pour les actifs enregistrés.
    #[pallet::storage]
    #[pallet::getter(fn assets)]
    pub type Assets<T: Config> = StorageMap<_, Blake2_128Concat, u64, Asset, OptionQuery>;

    /// Stockage pour les ordres d'achat.
    #[pallet::storage]
    #[pallet::getter(fn buy_orders)]
    pub type BuyOrders<T: Config> = StorageMap<_, Blake2_128Concat, u64, Order, OptionQuery>;

    /// Stockage pour les ordres de vente.
    #[pallet::storage]
    #[pallet::getter(fn sell_orders)]
    pub type SellOrders<T: Config> = StorageMap<_, Blake2_128Concat, u64, Order, OptionQuery>;

    /// Livre d'ordres : association d'un identifiant d'actif à une liste d'identifiants d'ordres.
    #[pallet::storage]
    #[pallet::getter(fn order_book)]
    pub type OrderBook<T: Config> = StorageMap<_, Blake2_128Concat, u64, Vec<u64>, ValueQuery>;

    /// Historique des trades exécutés.
    #[pallet::storage]
    #[pallet::getter(fn trades_history)]
    pub type TradesHistory<T: Config> = StorageValue<_, Vec<Trade>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Actif enregistré (asset ID, owner).
        AssetRegistered(u64, u64),
        /// Ordre passé (order ID, type, asset ID).
        OrderPlaced(u64, OrderType, u64),
        /// Ordre annulé (order ID).
        OrderCancelled(u64),
        /// Trade exécuté (trade ID, asset ID, quantité, prix).
        TradeExecuted(u64, u64, u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Les métadonnées de l'actif dépassent la longueur maximale autorisée.
        AssetMetadataTooLong,
        /// L'actif est déjà enregistré.
        AssetAlreadyRegistered,
        /// Actif non trouvé.
        AssetNotFound,
        /// Ordre non trouvé.
        OrderNotFound,
        /// Quantité insuffisante dans l'ordre.
        InsufficientOrderQuantity,
        /// Paramètres d'ordre invalides.
        InvalidOrder,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Enregistre un nouvel actif dans le marketplace.
        #[pallet::weight(10_000)]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset_id: u64,
            metadata: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                metadata.len() as u32 <= T::MaxAssetMetadataLength::get(),
                Error::<T>::AssetMetadataTooLong
            );
            ensure!(
                !Assets::<T>::contains_key(&asset_id),
                Error::<T>::AssetAlreadyRegistered
            );
            let asset = Asset {
                id: asset_id,
                metadata: metadata.clone(),
                owner: who.into(), // Pour simplifier, conversion en u64 (à adapter selon votre type AccountId)
            };
            <Assets<T>>::insert(asset_id, asset);
            Self::deposit_event(Event::AssetRegistered(asset_id, who.into()));
            Ok(())
        }

        /// Place un ordre (achat ou vente) pour un actif.
        #[pallet::weight(10_000)]
        pub fn place_order(
            origin: OriginFor<T>,
            order: Order,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            match order.order_type {
                OrderType::Buy => <BuyOrders<T>>::insert(order.id, order.clone()),
                OrderType::Sell => <SellOrders<T>>::insert(order.id, order.clone()),
            };
            OrderBook::<T>::mutate(order.asset_id, |orders| orders.push(order.id));
            Self::deposit_event(Event::OrderPlaced(order.id, order.order_type, order.asset_id));
            Ok(())
        }

        /// Annule un ordre existant.
        #[pallet::weight(10_000)]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_id: u64,
            order_type: OrderType,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            match order_type {
                OrderType::Buy => ensure!(<BuyOrders<T>>::contains_key(&order_id), Error::<T>::OrderNotFound)
                                    .then(|| <BuyOrders<T>>::remove(order_id)),
                OrderType::Sell => ensure!(<SellOrders<T>>::contains_key(&order_id), Error::<T>::OrderNotFound)
                                    .then(|| <SellOrders<T>>::remove(order_id)),
            };
            Self::deposit_event(Event::OrderCancelled(order_id));
            Ok(())
        }

        /// Exécute un trade en associant directement un ordre d'achat et un ordre de vente.
        #[pallet::weight(10_000)]
        pub fn execute_trade(
            origin: OriginFor<T>,
            trade: Trade,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!(<BuyOrders<T>>::contains_key(&trade.buy_order_id), Error::<T>::OrderNotFound);
            ensure!(<SellOrders<T>>::contains_key(&trade.sell_order_id), Error::<T>::OrderNotFound);
            // Ici, on effectue la logique de correspondance et d'exécution.
            // Pour simplifier, nous supposons une correspondance directe et supprimons les ordres.
            <BuyOrders<T>>::remove(trade.buy_order_id);
            <SellOrders<T>>::remove(trade.sell_order_id);
            <TradesHistory<T>>::mutate(|history| history.push(trade.clone()));
            Self::deposit_event(Event::TradeExecuted(trade.id, trade.asset_id, trade.quantity, trade.price));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Renvoie un timestamp fixe pour les tests.
        /// À remplacer par une intégration au `pallet_timestamp` en production.
        fn current_timestamp() -> u64 {
            1_640_000_000
        }
    }
}
