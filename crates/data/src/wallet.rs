use alloy::{
    hex,
    primitives::Address,
    signers::{Signer, local::PrivateKeySigner},
};
use std::str::FromStr;
use thiserror::Error;

/// Default message used for signing operations
pub const DEFAULT_SIGN_MESSAGE: &str = "Please sign to retrieve your encryption key";

/// Error enumeration for wallet operations
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Env Var error: {0}")]
    VarError(#[from] std::env::VarError),
}

/// Local Ethereum wallet structure.
///
/// Encapsulates Ethereum address and signer, providing:
/// - Initialization from private key
/// - Message signing capabilities
/// - Default message signing
#[derive(Debug, Clone)]
pub struct LocalEthWallet {
    /// Ethereum address derived from private key
    pub address: Address,
    /// Private key signer instance
    pub signer: PrivateKeySigner,
}

impl LocalEthWallet {
    /// Creates a new wallet instance from a private key string
    ///
    /// # Arguments
    /// * `private_key` - Hex-formatted private key string (with or without 0x prefix)
    ///
    /// # Errors
    /// Returns [`WalletError::SigningError`] if private key format is invalid
    pub fn new(private_key: impl AsRef<str>) -> Result<Self, WalletError> {
        let signer = PrivateKeySigner::from_str(private_key.as_ref())
            .map_err(|e| WalletError::SigningError(e.to_string()))?;
        let address = signer.address();
        Ok(Self { address, signer })
    }

    /// Creates a new wallet instance from a private key env var `PRIVATE_KEY`.
    #[inline]
    pub fn from_env() -> Result<Self, WalletError> {
        Self::new(std::env::var("PRIVATE_KEY")?)
    }

    /// Creates a new wallet instance from a random keypair seeded.
    #[inline]
    pub fn random() -> Result<Self, WalletError> {
        let signer = PrivateKeySigner::random();
        let address = signer.address();
        Ok(Self { address, signer })
    }

    /// Signs an arbitrary message
    ///
    /// # Arguments
    /// * `message` - Raw string message to sign
    ///
    /// # Returns
    /// Signature byte.
    #[inline]
    pub async fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, WalletError> {
        self.signer
            .sign_message(message)
            .await
            .map(|sig| sig.as_bytes().to_vec())
            .map_err(|e| WalletError::SigningError(e.to_string()))
    }

    /// Signs an arbitrary message
    ///
    /// # Arguments
    /// * `message` - Raw string message to sign
    ///
    /// # Returns
    /// Hex-encoded signature string
    #[inline]
    pub async fn sign_message_hex(&self, message: &[u8]) -> Result<String, WalletError> {
        self.signer
            .sign_message(message)
            .await
            .map(|sig| hex::encode(sig.as_bytes()))
            .map_err(|e| WalletError::SigningError(e.to_string()))
    }

    /// Signs the default message
    ///
    /// See [`DEFAULT_SIGN_MESSAGE`] for default message content
    #[inline]
    pub async fn sign_hex(&self) -> Result<String, WalletError> {
        self.sign_message_hex(DEFAULT_SIGN_MESSAGE.as_bytes()).await
    }

    /// Signs the default message
    ///
    /// See [`DEFAULT_SIGN_MESSAGE`] for default message content
    #[inline]
    pub async fn sign(&self) -> Result<Vec<u8>, WalletError> {
        self.sign_message(DEFAULT_SIGN_MESSAGE.as_bytes()).await
    }
}
