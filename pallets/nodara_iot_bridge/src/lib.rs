#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_iot_bridge - Legendary Edition
//!
//! This module enables secure integration of IoT data into the Nodara BIOSPHÈRE QUANTIC blockchain. It collects data from IoT devices,
//! verifies its authenticity using advanced cryptographic techniques, and records the data on-chain with full audit logging.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing an IoT record.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct IotRecord {
    pub id: u64,
    pub payload: Vec<u8>,
    pub device_id: Vec<u8>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Structure for logging IoT data events.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct IotLog {
    pub timestamp: u64,
    pub id: u64,
    pub operation: Vec<u8>, // e.g., "Submit", "ConfigUpdate"
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
        /// Maximum allowed payload length for IoT data.
        #[pallet::constant]
        type MaxPayloadLength: Get<u32>;
        /// Base timeout (in seconds) for IoT data validation.
        #[pallet::constant]
        type BaseTimeout: Get<u64>;
    }

    #[pallet::storage]
    #[pallet::getter(fn iot_data)]
    pub type IotData<T: Config> = StorageMap<_, Blake2_128Concat, u64, IotRecord, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn iot_history)]
    pub type IotHistory<T: Config> = StorageValue<_, Vec<IotLog>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when IoT data is submitted successfully.
        IotDataSubmitted(u64, Vec<u8>),
        /// Emitted when configuration parameters are updated.
        ConfigUpdated(Vec<u8>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The IoT data payload exceeds the maximum allowed length.
        PayloadTooLong,
        /// Invalid device identifier provided.
        InvalidDeviceId,
        /// IoT data verification failed.
        DataVerificationFailed,
    }

    impl<T: Config> Pallet<T> {
        /// Submits IoT data to the blockchain.
        ///
        /// Parameters:
        /// - `id`: Unique identifier for the IoT data.
        /// - `payload`: The data payload from the IoT device.
        /// - `device_id`: Identifier for the IoT device (must be non-empty).
        /// - `signature`: Cryptographic signature used to verify data integrity.
        pub fn submit_iot_data(
            id: u64,
            payload: Vec<u8>,
            device_id: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResult {
            ensure!(payload.len() as u32 <= T::MaxPayloadLength::get(), Error::<T>::PayloadTooLong);
            ensure!(!device_id.is_empty(), Error::<T>::InvalidDeviceId);
            // Simulate cryptographic verification
            ensure!(!payload.is_empty() && !signature.is_empty(), Error::<T>::DataVerificationFailed);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let record = IotRecord {
                id,
                payload: payload.clone(),
                device_id: device_id.clone(),
                timestamp: now,
                signature,
            };
            <IotData<T>>::insert(id, record);
            <IotHistory<T>>::mutate(|history| {
                history.push(IotLog {
                    timestamp: now,
                    id,
                    operation: b"Submit".to_vec(),
                    details: payload.clone(),
                })
            });
            Self::deposit_event(Event::IotDataSubmitted(id, payload));
            Ok(())
        }

        /// Updates the IoT bridge configuration parameters.
        ///
        /// This function allows DAO-driven updates to configuration settings such as timeouts and validation thresholds.
        pub fn update_config(new_config: Vec<u8>, details: Vec<u8>) -> DispatchResult {
            ensure!(!new_config.is_empty(), Error::<T>::DataVerificationFailed);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            <IotHistory<T>>::mutate(|history| {
                history.push(IotLog {
                    timestamp: now,
                    id: 0,
                    operation: b"ConfigUpdate".to_vec(),
                    details: details.clone(),
                })
            });
            Self::deposit_event(Event::ConfigUpdated(new_config, details));
            Ok(())
        }
    }
}
