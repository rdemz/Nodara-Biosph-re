#![cfg_attr(not(feature = "std"), no_std)]

//! # Nodara Runtime
//!
//! Assembly of the complete Nodara BIOSPHÈRE QUANTIC runtime.
//!
//! Ce fichier constitue une implémentation minimale du runtime, incluant les types de base, 
//! une définition d'un `RuntimeCall` dummy, ainsi qu'une déclaration d'API runtime minimale.

use sp_core::OpaqueMetadata;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
    MultiSignature, Perbill,
};
use sp_version::RuntimeVersion;
use parity_scale_codec::{Encode, Decode};

// Pour importer les API de runtime.
#[macro_use]
extern crate sp_api;

// ====================================================================
// Type definitions
// ====================================================================

/// Type utilisé pour les numéros de bloc.
pub type BlockNumber = u32;
/// Type utilisé pour les indices (nonce).
pub type Index = u32;
/// Type utilisé pour les soldes.
pub type Balance = u128;

/// Signature utilisée dans le runtime.
pub type Signature = MultiSignature;
/// Identifiant de compte.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Pour simplifier, on définit un type dummy pour SignedExtra.
/// Dans un runtime réel, il s'agira d'un tuple regroupant divers "signed extras".
pub type SignedExtra = ();

/// Type Header pour les blocs.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Définition d'un RuntimeCall minimal. Dans un vrai runtime, ce type est généré par `construct_runtime!`.
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum RuntimeCall {
    SystemCall,
    // Ajoutez ici d'autres appels.
}

/// Définition du type Block.
pub type Block = generic::Block<Header, RuntimeCall>;

/// Type d'extrinsèque non vérifié.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;

/// Type opaque d'extrinsèque (utilisé pour la métadonnée).
pub type OpaqueExtrinsic = sp_runtime::OpaqueExtrinsic;

/// Opaque metadata pour le runtime.
pub const OPAQUE_METADATA: OpaqueMetadata =
    OpaqueMetadata::new(OpaqueExtrinsic::default().encode());

// ====================================================================
// Runtime version
// ====================================================================

#[cfg(feature = "std")]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    impl_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
};

#[cfg(not(feature = "std"))]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    impl_name: sp_runtime::create_runtime_str!("nodara-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
};

// ====================================================================
// Dummy runtime API declarations
// ====================================================================

sp_api::decl_runtime_apis! {
    pub trait DummyApi {
        fn dummy() -> u32;
    }
}

/// Implémentation dummy de l'API runtime.
impl DummyApi for Runtime {
    fn dummy() -> u32 { 42 }
}

// ====================================================================
// Remarque : Construct Runtime
// ====================================================================
//
// Dans un runtime Substrate complet, vous utiliseriez le macro `construct_runtime!`
// pour assembler tous les pallets et générer automatiquement les types `RuntimeCall`,
// `SignedExtra`, et les APIs runtime. Cette version minimale sert d'exemple pour compiler
// le runtime avec les versions homogènes du workspace.
// Vous devrez adapter et étendre cette implémentation selon les besoins de votre projet.

/// Dummy runtime struct.
pub struct Runtime;
