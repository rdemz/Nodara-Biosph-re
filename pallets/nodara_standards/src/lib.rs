#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_standards - Legendary Edition
//!
//! This module defines and enforces technical and regulatory standards for Nodara BIOSPHÈRE QUANTIC.
//! It allows the network to verify that all operations comply with established protocols and regulations,
//! and maintains immutable audit logs for transparency. DAO governance is integrated for dynamic updates.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing a standard definition.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct Standard {
    pub id: Vec<u8>,
    pub description: Vec<u8>,
    pub parameters: Vec<u8>,
}

/// Structure for logging compliance events.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ComplianceLog {
    pub timestamp: u64,
    pub standard_id: Vec<u8>,
    pub operation_data: Vec<u8>,
    pub outcome: bool,
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
        /// Maximum allowed length for standard definitions (description + parameters).
        #[pallet::constant]
        type MaxStandardLength: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn standards)]
    pub type Standards<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Standard, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn compliance_history)]
    pub type ComplianceHistory<T: Config> = StorageValue<_, Vec<ComplianceLog>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when a new standard is defined.
        StandardDefined(Vec<u8>),
        /// Emitted when a standard is updated.
        StandardUpdated(Vec<u8>),
        /// Emitted when a compliance check is performed (standard id, outcome).
        ComplianceChecked(Vec<u8>, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The standard definition exceeds the maximum allowed length.
        StandardTooLong,
        /// The standard already exists.
        StandardAlreadyExists,
        /// The standard does not exist.
        StandardNotFound,
        /// Compliance check failed.
        ComplianceCheckFailed,
    }

    impl<T: Config> Pallet<T> {
        /// Defines a new standard.
        pub fn define_standard(origin: T::Origin, id: Vec<u8>, description: Vec<u8>, parameters: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!((description.len() + parameters.len()) as u32 <= T::MaxStandardLength::get(), Error::<T>::StandardTooLong);
            ensure!(!Standards::<T>::contains_key(&id), Error::<T>::StandardAlreadyExists);
            let standard = Standard { id: id.clone(), description, parameters };
            Standards::<T>::insert(&id, standard);
            Self::deposit_event(Event::StandardDefined(id));
            Ok(())
        }

        /// Updates an existing standard.
        pub fn update_standard(origin: T::Origin, id: Vec<u8>, new_description: Vec<u8>, new_parameters: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            ensure!((new_description.len() + new_parameters.len()) as u32 <= T::MaxStandardLength::get(), Error::<T>::StandardTooLong);
            Standards::<T>::try_mutate(&id, |maybe_standard| -> DispatchResult {
                let standard = maybe_standard.as_mut().ok_or(Error::<T>::StandardNotFound)?;
                standard.description = new_description;
                standard.parameters = new_parameters;
                Ok(())
            })?;
            Self::deposit_event(Event::StandardUpdated(id));
            Ok(())
        }

        /// Verifies compliance of a given operation against a defined standard.
        ///
        /// Returns `true` if the operation complies with the standard.
        pub fn verify_compliance(standard_id: Vec<u8>, operation_data: Vec<u8>) -> Result<bool, DispatchError> {
            let standard = Standards::<T>::get(&standard_id).ok_or(Error::<T>::StandardNotFound)?;
            // Simulate a compliance check by verifying that the operation_data contains the standard parameters.
            let complies = operation_data.windows(standard.parameters.len()).any(|window| window == standard.parameters.as_slice());
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            ComplianceHistory::<T>::mutate(|history| history.push(ComplianceLog {
                timestamp: now,
                standard_id: standard_id.clone(),
                operation_data: operation_data.clone(),
                outcome: complies,
            }));
            Self::deposit_event(Event::ComplianceChecked(standard_id, complies));
            Ok(complies)
        }
    }
}
