#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_id - Legendary Edition
//!
//! This module implements a decentralized identity management system for Nodara BIOSPHÈRE QUANTIC.
//! It allows secure registration and update of user identities along with continuous KYC verification.
//! All identity events are logged immutably to ensure transparency and compliance.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure for storing identity data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct IdentityData {
    pub kyc_details: Vec<u8>,  // Encrypted KYC details
    pub verified: bool,        // Verification status
}

/// Structure to log identity events.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct IdentityEvent<T: Config> {
    pub account: T::AccountId,
    pub timestamp: u64,
    pub previous_status: bool,
    pub new_status: bool,
    pub details: Vec<u8>,
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
        /// Maximum allowed length for KYC details.
        #[pallet::constant]
        type MaxKycLength: Get<u32>;
        /// Default verification status for new identities.
        #[pallet::constant]
        type DefaultVerification: Get<bool>;
    }

    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, IdentityData, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn identity_history)]
    pub type IdentityHistory<T: Config> = StorageValue<_, Vec<IdentityEvent<T>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when an identity is registered (account, details, verification status).
        IdentityRegistered(T::AccountId, Vec<u8>, bool),
        /// Emitted when an identity is updated (account, previous status, new status, details).
        IdentityUpdated(T::AccountId, bool, bool, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// KYC details exceed the maximum allowed length.
        KycTooLong,
        /// Identity already exists.
        IdentityAlreadyExists,
        /// Identity not found.
        IdentityNotFound,
    }

    impl<T: Config> Pallet<T> {
        /// Registers a new identity with the given KYC details.
        pub fn register_identity(origin: T::Origin, kyc_details: Vec<u8>) -> DispatchResult {
            let account = ensure_signed(origin)?;
            ensure!((kyc_details.len() as u32) <= T::MaxKycLength::get(), Error::<T>::KycTooLong);
            ensure!(!Identities::<T>::contains_key(&account), Error::<T>::IdentityAlreadyExists);

            let identity = IdentityData {
                kyc_details: kyc_details.clone(),
                verified: T::DefaultVerification::get(),
            };
            <Identities<T>>::insert(&account, identity);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let event = IdentityEvent {
                account: account.clone(),
                timestamp: now,
                previous_status: false,
                new_status: T::DefaultVerification::get(),
                details: kyc_details.clone(),
            };
            <IdentityHistory<T>>::mutate(|history| history.push(event));
            Self::deposit_event(Event::IdentityRegistered(account, kyc_details, T::DefaultVerification::get()));
            Ok(())
        }

        /// Updates an existing identity with new KYC details and a new verification status.
        pub fn update_identity(origin: T::Origin, new_kyc_details: Vec<u8>, new_verified: bool) -> DispatchResult {
            let account = ensure_signed(origin)?;
            ensure!((new_kyc_details.len() as u32) <= T::MaxKycLength::get(), Error::<T>::KycTooLong);
            Identities::<T>::try_mutate(&account, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                let previous_status = identity.verified;
                identity.kyc_details = new_kyc_details.clone();
                identity.verified = new_verified;
                let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
                let event = IdentityEvent {
                    account: account.clone(),
                    timestamp: now,
                    previous_status,
                    new_status: new_verified,
                    details: new_kyc_details.clone(),
                };
                <IdentityHistory<T>>::mutate(|history| history.push(event));
                Ok(())
            })?;
            Self::deposit_event(Event::IdentityUpdated(account, false, new_verified, new_kyc_details));
            Ok(())
        }
    }
}
