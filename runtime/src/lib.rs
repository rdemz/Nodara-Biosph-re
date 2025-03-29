#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara Runtime - Extreme Production-Ready Version
//!
//! Assembly of the complete Nodara BIOSPHÈRE QUANTIC runtime. This runtime includes system, timestamp,
//! and a dummy pallet (as an example) assembled via the `construct_runtime!` macro. It provides full
//! definitions for extrinsics, header, and runtime APIs. All dependencies are locked to ensure reproducibility.

use sp_core::OpaqueMetadata;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
    MultiSignature,
};
use sp_version::RuntimeVersion;
use parity_scale_codec::{Encode, Decode};

#[macro_use]
extern crate sp_api;

// ====================================================================
// Type Definitions
// ====================================================================

/// Block number type.
pub type BlockNumber = u32;
/// Nonce (index) type.
pub type Index = u32;
/// Balance type.
pub type Balance = u128;

/// Signature type.
pub type Signature = MultiSignature;
/// Account ID type.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// SignedExtra type for extrinsics.
/// In a production runtime, this tuple regroups various signed extras.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckMortality<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
);

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, RuntimeCall>;
/// Unchecked extrinsic type.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;

/// Opaque extrinsic type (used for metadata).
pub type OpaqueExtrinsic = sp_runtime::OpaqueExtrinsic;

/// Opaque metadata for the runtime.
pub const OPAQUE_METADATA: OpaqueMetadata =
    OpaqueMetadata::new(OpaqueExtrinsic::default().encode());

// ====================================================================
// Runtime Version
// ====================================================================

#[cfg(feature = "std")]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    impl_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    authoring_version: 1,
    spec_version: 2, // Version avancée
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
};

#[cfg(not(feature = "std"))]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    impl_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    authoring_version: 1,
    spec_version: 2,
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
};

// ====================================================================
// Construct Runtime
// ====================================================================

frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        // Dummy module for demonstration purposes.
        DummyModule: dummy_pallet::{Pallet, Call, Storage, Event<T>},
    }
);

// ====================================================================
// Dummy Pallet (Example)
// ====================================================================

pub mod dummy_pallet {
    use frame_support::{_
