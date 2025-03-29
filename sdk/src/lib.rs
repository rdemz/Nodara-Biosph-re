#![cfg_attr(not(feature = "std"), no_std)]

// Expose the alloc crate when no_std is active.
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// # Nodara SDK Library - Extreme Version
///
/// This SDK provides a comprehensive set of tools for interacting with the Nodara BIOSPHÈRE QUANTIC network.
/// It includes functions for network connection, transaction construction and submission, cryptographic utilities,
/// and data encoding/decoding. Designed to be fully deployable in no_std environments, it forms the basis for building
/// advanced applications on the Nodara network.

pub mod error {
    use core::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub enum SdkError {
        ConnectionFailed,
        TransactionSubmissionFailed,
        BalanceQueryFailed,
        SignatureVerificationFailed,
        EncodingError,
        DecodingError,
    }

    impl fmt::Display for SdkError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SdkError::ConnectionFailed => write!(f, "Connection to Nodara network failed"),
                SdkError::TransactionSubmissionFailed => write!(f, "Transaction submission failed"),
                SdkError::BalanceQueryFailed => write!(f, "Balance query failed"),
                SdkError::SignatureVerificationFailed => write!(f, "Signature verification failed"),
                SdkError::EncodingError => write!(f, "Data encoding error"),
                SdkError::DecodingError => write!(f, "Data decoding error"),
            }
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for SdkError {}
}

pub mod crypto {
    use parity_scale_codec::{Decode, Encode};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PublicKey(pub [u8; 32]);

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PrivateKey(pub [u8; 32]);

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct KeyPair {
        pub public: PublicKey,
        pub private: PrivateKey,
    }

    impl KeyPair {
        /// Dummy key generation (replace with a secure RNG in production).
        pub fn generate() -> Self {
            Self {
                public: PublicKey([1u8; 32]),
                private: PrivateKey([2u8; 32]),
            }
        }

        /// Dummy signing function: retourne la donnée à laquelle on ajoute un byte fixe.
        pub fn sign(&self, message: &[u8]) -> Vec<u8> {
            let mut sig = message.to_vec();
            sig.push(0xAA);
            sig
        }
    }

    /// Dummy signature verification: vérifie que la signature est bien le message suivi de 0xAA.
    pub fn verify(message: &[u8], signature: &[u8], _public: &PublicKey) -> bool {
        let mut expected = message.to_vec();
        expected.push(0xAA);
        signature == expected
    }
}

pub mod transaction {
    use parity_scale_codec::{Decode, Encode};

    #[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
    pub struct Transaction {
        pub from: u64,
        pub to: u64,
        pub amount: u128,
        pub payload: Vec<u8>,
    }

    impl Transaction {
        pub fn new(from: u64, to: u64, amount: u128, payload: Vec<u8>) -> Self {
            Self { from, to, amount, payload }
        }
    }
}

pub mod client {
    use crate::error::SdkError;
    use crate::transaction::Transaction;

    /// Dummy client for interacting with the Nodara network.
    pub struct NodaraSdk {
        connected: bool,
    }

    impl NodaraSdk {
        /// Crée un nouveau client Nodara.
        pub fn new() -> Self {
            Self { connected: false }
        }

        /// Simule la connexion au réseau Nodara.
        pub fn connect(&mut self) -> Result<(), SdkError> {
            self.connected = true;
            Ok(())
        }

        /// Simule la soumission d'une transaction.
        pub fn submit_transaction(&self, tx: Transaction) -> Result<(), SdkError> {
            if !self.connected {
                return Err(SdkError::ConnectionFailed);
            }
            // Ici, on encoderait et signerait la transaction pour la soumettre.
            Ok(())
        }

        /// Simule une requête de solde pour un compte donné.
        pub fn query_balance(&self, _account: u64) -> Result<u128, SdkError> {
            if !self.connected {
                return Err(SdkError::ConnectionFailed);
            }
            // Pour la démonstration, on retourne une valeur fixe.
            Ok(1_000_000)
        }
    }
}

pub mod utils {
    use parity_scale_codec::{Decode, Encode};

    /// Encode les données en utilisant SCALE.
    pub fn encode_data<T: Encode>(data: &T) -> Vec<u8> {
        data.encode()
    }

    /// Décode les données en utilisant SCALE.
    pub fn decode_data<T: Decode>(data: &[u8]) -> Result<T, parity_scale_codec::Error> {
        T::decode(&mut &data[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn dummy_sdk_functionality() {
        // Test de la connexion et de la soumission d'une transaction.
        let mut sdk = client::NodaraSdk::new();
        assert!(sdk.connect().is_ok());
        let tx = transaction::Transaction::new(1, 2, 1000, vec![1, 2, 3]);
        assert!(sdk.submit_transaction(tx).is_ok());
        let balance = sdk.query_balance(1);
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 1_000_000);

        // Test des fonctions cryptographiques.
        let message = b"Test message";
        let keypair = crypto::KeyPair::generate();
        let signature = keypair.sign(message);
        assert!(crypto::verify(message, &signature, &keypair.public));

        // Test des utilitaires d'encodage/décodage.
        let encoded = utils::encode_data(&"Hello Nodara".to_string());
        let decoded: String = utils::decode_data(&encoded).unwrap();
        assert_eq!(decoded, "Hello Nodara".to_string());
    }
}
