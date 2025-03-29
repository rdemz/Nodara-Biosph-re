#![cfg_attr(not(feature = "std"), no_std)]

//! # Legal and Compliance Module - Nodara BIOSPHÈRE QUANTIC
//!
//! This module enforces regulatory and legal constraints on-chain. It allows:
//! - Verification and update of compliance scores per account
//! - Logging of compliance events for audits
//! - Enforcing a minimum compliance level for access
//!
//! ## Storage
//! - `ComplianceStatus`: Current compliance score of each account
//! - `ComplianceHistory`: Audit log of all compliance updates
//!
//! ## Events
//! - `ComplianceUpdated`: Triggered when an account's compliance level is changed

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Minimum compliance level required.
        #[pallet::constant]
        type MinComplianceLevel: Get<u32>;
    }

    /// Stores the compliance score of each account.
    #[pallet::storage]
    #[pallet::getter(fn compliance_status)]
    pub type ComplianceStatus<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// History log of compliance updates for auditability.
    #[pallet::storage]
    #[pallet::getter(fn compliance_history)]
    pub type ComplianceHistory<T: Config> =
        StorageValue<_, Vec<(T::AccountId, u32, u64)>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A compliance score was updated: (account, new_score)
        ComplianceUpdated(T::AccountId, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Submitted compliance level is below the minimum threshold.
        ComplianceLevelTooLow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update caller’s compliance score.
        ///
        /// Will be rejected if score is below `MinComplianceLevel`.
        #[pallet::weight(10_000)]
        pub fn update_compliance_status(
            origin: OriginFor<T>,
            new_score: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                new_score >= T::MinComplianceLevel::get(),
                Error::<T>::ComplianceLevelTooLow
            );

            ComplianceStatus::<T>::insert(&who, new_score);

            let now = Self::current_timestamp();
            ComplianceHistory::<T>::mutate(|log| log.push((who.clone(), new_score, now)));

            Self::deposit_event(Event::ComplianceUpdated(who, new_score));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Returns current timestamp (stub: replace with actual timestamp logic).
        fn current_timestamp() -> u64 {
            1_700_000_000 // Replace with runtime timestamp logic if needed
        }
    }
}
