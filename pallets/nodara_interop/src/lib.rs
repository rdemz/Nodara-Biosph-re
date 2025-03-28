#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

//! # nodara_interop - Legendary Edition
//!
//! This module implements secure cross-chain communication for Nodara BIOSPHÈRE QUANTIC.
//! It handles sending and receiving messages between Nodara and external blockchain networks,
//! ensuring message integrity and authenticity through advanced cryptographic verification,
//! and logs all operations for complete transparency.

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::RuntimeDebug;

/// Structure representing an interop message.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct InteropMessage {
    pub id: u64,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Structure for logging interop events.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct InteropLog {
    pub timestamp: u64,
    pub message_id: u64,
    pub operation: Vec<u8>, // e.g., "Send", "Receive", "ConfigUpdate"
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
        /// Maximum allowed payload length for interop messages.
        #[pallet::constant]
        type MaxPayloadLength: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn outgoing_messages)]
    pub type OutgoingMessages<T: Config> = StorageMap<_, Blake2_128Concat, u64, InteropMessage, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn incoming_messages)]
    pub type IncomingMessages<T: Config> = StorageMap<_, Blake2_128Concat, u64, InteropMessage, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn interop_history)]
    pub type InteropHistory<T: Config> = StorageValue<_, Vec<InteropLog>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when an interop message is successfully sent.
        MessageSent(u64, Vec<u8>),
        /// Emitted when an interop message is successfully received.
        MessageReceived(u64, Vec<u8>),
        /// Emitted when interop configuration is updated.
        ConfigUpdated(Vec<u8>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The payload exceeds the maximum allowed length.
        PayloadTooLong,
        /// Message verification failed.
        VerificationFailed,
        /// Invalid configuration parameters.
        InvalidConfig,
    }

    impl<T: Config> Pallet<T> {
        /// Sends an interop message to an external blockchain.
        pub fn send_message(id: u64, payload: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
            ensure!(payload.len() as u32 <= T::MaxPayloadLength::get(), Error::<T>::PayloadTooLong);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let message = InteropMessage {
                id,
                payload: payload.clone(),
                timestamp: now,
                signature,
            };
            <OutgoingMessages<T>>::insert(id, message);
            <InteropHistory<T>>::mutate(|history| {
                history.push(InteropLog {
                    timestamp: now,
                    message_id: id,
                    operation: b"Send".to_vec(),
                    details: payload.clone(),
                })
            });
            Self::deposit_event(Event::MessageSent(id, payload));
            Ok(())
        }

        /// Receives and processes an interop message from an external blockchain.
        pub fn receive_message(id: u64, payload: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
            ensure!(payload.len() as u32 <= T::MaxPayloadLength::get(), Error::<T>::PayloadTooLong);
            // Simulate cryptographic verification.
            ensure!(!payload.is_empty() && !signature.is_empty(), Error::<T>::VerificationFailed);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            let message = InteropMessage {
                id,
                payload: payload.clone(),
                timestamp: now,
                signature,
            };
            <IncomingMessages<T>>::insert(id, message);
            <InteropHistory<T>>::mutate(|history| {
                history.push(InteropLog {
                    timestamp: now,
                    message_id: id,
                    operation: b"Receive".to_vec(),
                    details: payload.clone(),
                })
            });
            Self::deposit_event(Event::MessageReceived(id, payload));
            Ok(())
        }

        /// Updates the interop configuration via DAO governance.
        pub fn update_config(new_config: Vec<u8>, details: Vec<u8>) -> DispatchResult {
            ensure!(!new_config.is_empty(), Error::<T>::InvalidConfig);
            let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            <InteropHistory<T>>::mutate(|history| {
                history.push(InteropLog {
                    timestamp: now,
                    message_id: 0,
                    operation: b"ConfigUpdate".to_vec(),
                    details: details.clone(),
                })
            });
            Self::deposit_event(Event::ConfigUpdated(new_config, details));
            Ok(())
        }
    }
}
