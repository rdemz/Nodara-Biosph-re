#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara BIOSPHÈRE QUANTIC Runtime - Legendary Edition
//!
//! This runtime forms the backbone of Nodara BIOSPHÈRE QUANTIC. It integrates all the core pallets
//! (Biosphere, GrowthModel, StabilityGuard, LiquidityFlow, ReserveFund, RewardEngine, ID, Marketplace,
//! IoTBridge, Interop, PredictiveGuard, Reputation, Standards, POW) along with a dedicated Bridge module
//! for inter-chain interoperability. Built on Substrate, it leverages advanced security and performance
//! optimizations to ensure that the network operates at legendary levels.

use sp_runtime::{
    traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
    create_runtime_str, generic, RuntimeVersion,
};
use sp_core::OpaqueMetadata;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{KeyOwnerProofSystem, Randomness},
};
use frame_system as system;

// Basic types used by the runtime
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, RuntimeCall, signature::Signature, SignedExtra>;
pub type Block = generic::Block<Header, RuntimeCall>;
pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Hash = sp_core::H256;

// Runtime version
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("nodara-biosphere-quantic"),
    impl_name: create_runtime_str!("nodara-biosphere-quantic"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    transaction_version: 1,
    apis: sp_api::impl_runtime_apis!([]),
};

// Parameter types for basic runtime settings
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const MaximumBlockWeight: u32 = 2 * 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024 * 1024;
    pub const AvailableBlockRatio: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);
}

// Construct the runtime by including all the pallets
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // System support
        System: system::{Pallet, Call, Config, Storage, Event<T>},

        // Timestamp for block time management
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},

        // Nodara core modules
        Biosphere: nodara_biosphere::{Pallet, Call, Storage, Event<T>},
        GrowthModel: nodara_growth_model::{Pallet, Call, Storage, Event<T>},
        StabilityGuard: nodara_stability_guard::{Pallet, Call, Storage, Event<T>},
        LiquidityFlow: nodara_liquidity_flow::{Pallet, Call, Storage, Event<T>},
        ReserveFund: nodara_reserve_fund::{Pallet, Call, Storage, Event<T>},
        RewardEngine: nodara_reward_engine::{Pallet, Call, Storage, Event<T>},
        ID: nodara_id::{Pallet, Call, Storage, Event<T>},
        Marketplace: nodara_marketplace::{Pallet, Call, Storage, Event<T>},
        IoTBridge: nodara_iot_bridge::{Pallet, Call, Storage, Event<T>},
        Interop: nodara_interop::{Pallet, Call, Storage, Event<T>},
        PredictiveGuard: nodara_predictive_guard::{Pallet, Call, Storage, Event<T>},
        Reputation: nodara_reputation::{Pallet, Call, Storage, Event<T>},
        Standards: nodara_standards::{Pallet, Call, Storage, Event<T>},
        POW: nodara_pow::{Pallet, Call, Storage, Event<T>},

        // *** Nouveau module Bridge Inter‑chaînes intégré ***
        Bridge: pallet_bridge::{Pallet, Call, Storage, Event<T>},
    }
);

// Opaque types for block construction
pub mod opaque {
    pub use super::Block;
    pub use sp_runtime::generic::Header;
    pub type BlockId = sp_runtime::generic::BlockId<Block>;
    pub type Signature = sp_runtime::MultiSignature;
}

// Metadata for the runtime (used for node interface)
pub const OPAQUE_METADATA: OpaqueMetadata = OpaqueMetadata::new(sp_runtime::OpaqueExtrinsic::default().encode());
