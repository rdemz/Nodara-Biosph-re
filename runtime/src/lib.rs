// runtime/src/lib.rs - Nodara BIOSPHÈRE QUANTIC Runtime (Production Ready)

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # Nodara BIOSPHÈRE QUANTIC Runtime
//!
//! Ce runtime intègre tous les modules du réseau Nodara : System, Timestamp, Aura, Grandpa, Session,
//! ainsi que tous les modules personnalisés : Bridge, Biosphere, Growth, Identity, Interop, IoTBridge,
//! LiquidityFlow, RewardEngine, StabilityGuard, Standards, Pow, PredictiveGuard, Reputation, ReserveFund,
//! Marketplace.
//!
//! Le runtime expose également une API complète (NodeRuntimeApi) pour interroger l'état des différents modules.

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

// ---------------------------------------------------------------------
// Type Definitions
// ---------------------------------------------------------------------

/// Block number.
pub type BlockNumber = u32;
/// Nonce.
pub type Index = u32;
/// Balance.
pub type Balance = u128;

/// Signature.
pub type Signature = MultiSignature;
/// AccountId.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// SignedExtra for extrinsics.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    pallet_timestamp::CheckTimestamp<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
);

/// Header.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block.
pub type Block = generic::Block<Header, RuntimeCall>;
/// Unchecked extrinsic.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;

/// Opaque extrinsic for metadata.
pub type OpaqueExtrinsic = sp_runtime::OpaqueExtrinsic;
pub const OPAQUE_METADATA: OpaqueMetadata =
    OpaqueMetadata::new(OpaqueExtrinsic::default().encode());

// ---------------------------------------------------------------------
// Runtime Version
// ---------------------------------------------------------------------

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    impl_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
};

// ---------------------------------------------------------------------
// Pallets Inclusion
// ---------------------------------------------------------------------

// Pallets standard.
pub mod system;
pub mod pallet_timestamp;
pub mod pallet_aura;
pub mod pallet_grandpa;
pub mod pallet_session;

// Vos modules personnalisés.
pub mod pallet_bridge;
pub mod nodara_biosphere;
pub mod nodara_growth;
pub mod nodara_id;
pub mod nodara_interop;
pub mod nodara_iot;
pub mod nodara_liquidity_flow;
pub mod nodara_reward_engine;
pub mod nodara_stability_guard;
pub mod nodara_standards;
pub mod nodara_pow;
pub mod nodara_predictive_guard;
pub mod nodara_reputation;
pub mod nodara_reserve_fund;
pub mod nodara_marketplace;

// ---------------------------------------------------------------------
// Construct Runtime!
// ---------------------------------------------------------------------

frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // Pallet système.
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Aura: pallet_aura::{Pallet, Call, Storage, Inherent, ValidateUnsigned},
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config as SessionConfig},

        // Vos modules personnalisés.
        Bridge: pallet_bridge::{Pallet, Call, Storage, Event<T>},
        Biosphere: nodara_biosphere::{Pallet, Call, Storage, Event<T>},
        Growth: nodara_growth::{Pallet, Call, Storage, Event<T>},
        Identity: nodara_id::{Pallet, Call, Storage, Event<T>},
        Interop: nodara_interop::{Pallet, Call, Storage, Event<T>},
        IoTBridge: nodara_iot::{Pallet, Call, Storage, Event<T>},
        LiquidityFlow: nodara_liquidity_flow::{Pallet, Call, Storage, Event<T>},
        RewardEngine: nodara_reward_engine::{Pallet, Call, Storage, Event<T>},
        StabilityGuard: nodara_stability_guard::{Pallet, Call, Storage, Event<T>},
        Standards: nodara_standards::{Pallet, Call, Storage, Event<T>},
        Pow: nodara_pow::{Pallet, Call, Storage, Event<T>},
        PredictiveGuard: nodara_predictive_guard::{Pallet, Call, Storage, Event<T>},
        Reputation: nodara_reputation::{Pallet, Call, Storage, Event<T>},
        ReserveFund: nodara_reserve_fund::{Pallet, Call, Storage, Event<T>},
        Marketplace: nodara_marketplace::{Pallet, Call, Storage, Event<T>},
    }
);

// ---------------------------------------------------------------------
// Configuration for Core Pallets
// ---------------------------------------------------------------------

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    // A typical minimum period is half the expected block time (e.g. 500ms if 1 second block time).
    type MinimumPeriod = ();
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = pallet_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
    type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
    type Event = RuntimeEvent;
    type Call = RuntimeCall;
    type WeightInfo = ();
}

impl pallet_session::Config for Runtime {
    type SessionManager = ();
    type Keys = pallet_aura::sr25519::AuthorityId; // Pour simplifier, on utilise Aura comme clé de session.
    type ShouldEndSession = ();
    type SessionHandler = ();
    type Event = RuntimeEvent;
    type ValidatorId = AccountId;
    type ValidatorIdOf = ();
    type WeightInfo = ();
}

// ---------------------------------------------------------------------
// Runtime API
// ---------------------------------------------------------------------

sp_api::decl_runtime_apis! {
    pub trait NodeRuntimeApi {
        /// Returns asset metadata (as bytes) for a given asset ID from the Marketplace module.
        fn marketplace_get_asset(asset_id: u64) -> Option<Vec<u8>>;

        /// Returns the global state of the Biosphere module.
        fn biosphere_get_state() -> nodara_biosphere::BioState;

        /// Returns the current growth state from the Growth module.
        fn growth_get_state() -> nodara_growth::GrowthState;

        /// Returns the identity data for a given account from the Identity module.
        fn identity_get(account: u64) -> Option<nodara_id::IdentityData>;

        /// Returns the interop history from the Interop module.
        fn interop_get_history() -> Vec<(u64, u64, Vec<u8>, Vec<u8>)>;

        /// Returns the IoT record for a given message ID from the IoT Bridge module.
        fn iot_get_record(message_id: u64) -> Option<nodara_iot::IotRecord>;

        /// Returns the liquidity state from the Liquidity Flow module.
        fn liquidity_get_state() -> nodara_liquidity_flow::LiquidityState;

        /// Returns the reward engine state from the Reward Engine module.
        fn reward_get_state() -> nodara_reward_engine::RewardEngineState<u64>;

        /// Returns the stability state from the Stability Guard module.
        fn stability_get_state() -> nodara_stability_guard::StabilityState;

        /// Returns the standard for a given ID from the Standards module.
        fn standards_get_standard(standard_id: Vec<u8>) -> Option<nodara_standards::Standard>;

        /// Returns the PoW state from the Pow module.
        fn pow_get_state() -> nodara_pow::PowState;

        /// Returns the current predictive value from the Predictive Guard module.
        fn predictive_get_value() -> u32;

        /// Returns the reputation record for a given account from the Reputation module.
        fn reputation_get(account: u64) -> Option<nodara_reputation::ReputationRecord>;

        /// Returns the reserve fund state from the Reserve Fund module.
        fn reserve_get_state() -> nodara_reserve_fund::ReserveFundState;

        /// Dummy function for testing.
        fn dummy() -> u32;
    }
}

impl NodeRuntimeApi for Runtime {
    fn marketplace_get_asset(asset_id: u64) -> Option<Vec<u8>> {
        nodara_marketplace::Pallet::<Runtime>::assets(asset_id).map(|asset| asset.metadata)
    }

    fn biosphere_get_state() -> nodara_biosphere::BioState {
        nodara_biosphere::Pallet::<Runtime>::bio_state()
    }

    fn growth_get_state() -> nodara_growth::GrowthState {
        nodara_growth::Pallet::<Runtime>::growth_state()
    }

    fn identity_get(account: u64) -> Option<nodara_id::IdentityData> {
        nodara_id::Pallet::<Runtime>::identities(account)
    }

    fn interop_get_history() -> Vec<(u64, u64, Vec<u8>, Vec<u8>)> {
        nodara_interop::Pallet::<Runtime>::interop_history()
    }

    fn iot_get_record(message_id: u64) -> Option<nodara_iot::IotRecord> {
        nodara_iot::Pallet::<Runtime>::iot_data(message_id)
    }

    fn liquidity_get_state() -> nodara_liquidity_flow::LiquidityState {
        nodara_liquidity_flow::Pallet::<Runtime>::liquidity_state()
    }

    fn reward_get_state() -> nodara_reward_engine::RewardEngineState<u64> {
        nodara_reward_engine::Pallet::<Runtime>::reward_engine_state()
    }

    fn stability_get_state() -> nodara_stability_guard::StabilityState {
        nodara_stability_guard::Pallet::<Runtime>::stability_state()
    }

    fn standards_get_standard(standard_id: Vec<u8>) -> Option<nodara_standards::Standard> {
        nodara_standards::Pallet::<Runtime>::standards(standard_id)
    }

    fn pow_get_state() -> nodara_pow::PowState {
        nodara_pow::Pallet::<Runtime>::pow_state()
    }

    fn predictive_get_value() -> u32 {
        nodara_predictive_guard::Pallet::<Runtime>::predictive_value()
    }

    fn reputation_get(account: u64) -> Option<nodara_reputation::ReputationRecord> {
        nodara_reputation::Pallet::<Runtime>::reputations(account)
    }

    fn reserve_get_state() -> nodara_reserve_fund::ReserveFundState {
        nodara_reserve_fund::Pallet::<Runtime>::reserve_fund_state()
    }

    fn dummy() -> u32 {
        42
    }
}

// ---------------------------------------------------------------------
// Runtime Struct
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runtime;
