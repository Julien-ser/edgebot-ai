//! EdgeBot AI License Verification System
//! 
//! This crate provides license verification for pro features using Ed25519 digital signatures.
//! Free core SDK remains open source under MIT/Apache-2.0.
//!
//! ## Usage
//! ```rust
//! use edgebot_licensing::{verify_pro_access, LicenseError};
//! 
//! fn main() -> Result<(), LicenseError> {
//!     verify_pro_access()?;
//!     // Pro features enabled
//!     Ok(())
//! }
//! ```
//! 
//! ## License Key Format
//! License keys are base64-encoded Ed25519 signatures over:
//! ```text
//! <customer_id>:<timestamp>:<features>
//! ```
//! Where:
//! - `customer_id`: Unique customer identifier
//! - `timestamp`: Unix timestamp (seconds)
//! - `features`: Comma-separated feature list (e.g., "cloud_sim,optimization")

use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey, SigningKey};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use time::macros::format_description;

/// Error types for licensing operations
#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("License key not found. Set EDGEBOT_LICENSE_KEY environment variable.")]
    LicenseKeyNotFound,
    
    #[error("Invalid license key format: {0}")]
    InvalidFormat(String),
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("License expired: {0}")]
    LicenseExpired(String),
    
    #[error("Feature '{0}' is not available in your license. Contact sales@edgebot.ai to upgrade.")]
    FeatureNotAvailable(String),
    
    #[error("Invalid public key configuration")]
    InvalidPublicKey,
}

/// License payload that is signed by EdgeBot AI licensing server
#[derive(Debug, Serialize, Deserialize)]
struct LicensePayload {
    customer_id: String,
    timestamp: i64,
    features: Vec<String>,
    expiry: Option<i64>,
}

/// Public key for verifying licenses
/// This is the Ed25519 public key that corresponds to the signing key on the licensing server.
/// IN PRODUCTION: This would be loaded from a secure configuration file or embedded at build time.
const PUBLIC_KEY_BASE64: &str = "YOUR_PUBLIC_KEY_HERE"; // TODO: Replace with actual public key

/// Get the embedded public key for signature verification
/// 
/// In production, this could be:
/// - Loaded from a secure system location
/// - Embedded at build time via build script
/// - Fetched from a trusted server on first run
fn get_public_key() -> Result<VerifyingKey, LicenseError> {
    let bytes = base64::decode(PUBLIC_KEY_BASE64)
        .map_err(|e| LicenseError::InvalidFormat(format!("Base64 decode failed: {}", e)))?;
    
    VerifyingKey::from_bytes(&bytes)
        .map_err(|_| LicenseError::InvalidPublicKey)
}

/// Parse and verify a license key
/// 
/// License key format: base64( signature || ":" || base64(payload) )
/// 
/// Arguments:
/// - `license_key`: The full license key string from environment variable
/// - `required_feature`: Optional feature to check availability for
/// 
/// Returns Ok(()) if license is valid and (if specified) contains the required feature.
pub fn verify_pro_access(required_feature: Option<&str>) -> Result<(), LicenseError> {
    // Get license key from environment
    let license_key = env::var("EDGEBOT_LICENSE_KEY")
        .map_err(|_| LicenseError::LicenseKeyNotFound)?;
    
    // Split signature and payload
    let parts: Vec<&str> = license_key.split(':').collect();
    if parts.len() != 2 {
        return Err(LicenseError::InvalidFormat(
            "License key must be in format: signature:payload".to_string()
        ));
    }
    
    let signature_bytes = base64::decode(parts[0])
        .map_err(|e| LicenseError::InvalidFormat(format!("Invalid base64 signature: {}", e)))?;
    
    let payload_bytes = base64::decode(parts[1])
        .map_err(|e| LicenseError::InvalidFormat(format!("Invalid base64 payload: {}", e)))?;
    
    // Parse signature
    let sig_bytes: [u8; 64] = signature_bytes.try_into()
        .map_err(|_| LicenseError::InvalidFormat("Invalid signature length".to_string()))?;
    let signature = Signature::from_bytes(&sig_bytes)
        .map_err(|_| LicenseError::InvalidFormat("Invalid signature format".to_string()))?;
    
    // Parse payload
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| LicenseError::InvalidFormat(format!("Invalid payload JSON: {}", e)))?;
    
    // Check expiry if present
    if let Some(expiry) = payload.expiry {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        if now > expiry {
            return Err(LicenseError::LicenseExpired(
                format!("License expired at {}", 
                    time::OffsetDateTime::from_unix_timestamp(expiry)
                        .map(|dt| dt.format(&format_description!("%Y-%m-%d %H:%M:%S UTC")).unwrap_or_else(|_| expiry.to_string()))
                        .unwrap_or_else(|_| expiry.to_string())
                )
            ));
        }
    }
    
    // Verify signature using public key
    let public_key = get_public_key()?;
    public_key.verify(&payload_bytes, &signature)
        .map_err(|_| LicenseError::SignatureVerificationFailed)?;
    
    // Check required feature if specified
    if let Some(feature) = required_feature {
        if !payload.features.contains(&feature.to_string()) {
            return Err(LicenseError::FeatureNotAvailable(feature.to_string()));
        }
    }
    
    Ok(())
}

/// Check if cloud simulation feature is available
/// 
/// Returns Ok(()) if license permits cloud simulation
/// 
/// # Example
/// ```rust
/// use edgebot_licensing::check_cloud_sim;
/// 
/// match check_cloud_sim() {
///     Ok(()) => println!("Cloud simulation available"),
///     Err(e) => eprintln!("Cloud simulation requires pro license: {}", e),
/// }
/// ```
pub fn check_cloud_sim() -> Result<(), LicenseError> {
    verify_pro_access(Some("cloud_sim"))
}

/// Check if optimization feature is available
/// 
/// Returns Ok(()) if license permits model optimization
/// 
/// # Example
/// ```rust
/// use edgebot_licensing::check_optimization;
/// 
/// match check_optimization() {
///     Ok(()) => println!("Optimization available"),
///     Err(e) => eprintln!("Optimization requires pro license: {}", e),
/// }
/// ```
pub fn check_optimization() -> Result<(), LicenseError> {
    verify_pro_access(Some("optimization"))
}

/// Utility function: generate a license key for development/testing
/// 
/// WARNING: This is for development purposes only. Do not use in production.
#[cfg(debug_assertions)]
pub fn generate_dev_license(
    customer_id: &str,
    features: Vec<&str>,
    secret_key_base64: &str,
) -> Result<String, LicenseError> {
    use base64::{engine::general_purpose, Engine as _};
    
    let secret_bytes = base64::decode(secret_key_base64)
        .map_err(|e| LicenseError::InvalidFormat(format!("Invalid secret key base64: {}", e)))?;
    
    let secret_key = SigningKey::from_bytes(&secret_bytes)
        .map_err(|_| LicenseError::InvalidFormat("Invalid secret key length".to_string()))?;
    
    let payload = LicensePayload {
        customer_id: customer_id.to_string(),
        timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
        features: features.iter().map(|s| s.to_string()).collect(),
        expiry: Some(time::OffsetDateTime::now_utc().unix_timestamp() + 365 * 24 * 3600), // 1 year
    };
    
    let payload_json = serde_json::to_vec(&payload)
        .map_err(|e| LicenseError::InvalidFormat(format!("Failed to serialize payload: {}", e)))?;
    
    let signature = secret_key.sign(&payload_json);
    
    let license_key = format!(
        "{}:{}",
        general_purpose::STANDARD.encode(signature.to_bytes()),
        general_purpose::STANDARD.encode(payload_json)
    );
    
    Ok(license_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_license_missing() {
        // Temporarily clear env var
        let original = env::var("EDGEBOT_LICENSE_KEY").ok();
        env::set_var("EDGEBOT_LICENSE_KEY", "");
        
        let result = verify_pro_access(None);
        assert!(matches!(result, Err(LicenseError::LicenseKeyNotFound)));
        
        // Restore
        if let Some(val) = original {
            env::set_var("EDGEBOT_LICENSE_KEY", val);
        } else {
            env::remove_var("EDGEBOT_LICENSE_KEY");
        }
    }
    
    #[test]
    fn test_license_invalid_format() {
        let original = env::var("EDGEBOT_LICENSE_KEY").ok();
        env::set_var("EDGEBOT_LICENSE_KEY", "invalid:format:too:many");
        
        let result = verify_pro_access(None);
        assert!(matches!(result, Err(LicenseError::InvalidFormat(_))));
        
        if let Some(val) = original {
            env::set_var("EDGEBOT_LICENSE_KEY", val);
        } else {
            env::remove_var("EDGEBOT_LICENSE_KEY");
        }
    }
    
    #[test]
    #[cfg(debug_assertions)]
    fn test_dev_license_generation() {
        // This test generates a dev license with a test key
        use base64::{engine::general_purpose, Engine as _};
        
        // Generate a test key pair
        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = VerifyingKey::from(&signing_key);
        
        let secret_key_base64 = general_purpose::STANDARD.encode(signing_key.to_bytes());
        let features = vec!["cloud_sim", "optimization"];
        
        let license = generate_dev_license("test_customer", features.clone(), &secret_key_base64)
            .expect("Failed to generate dev license");
        
        // Verify we can parse and validate it (but we'd need to set PUBLIC_KEY_CONSTANT)
        // This is limited in a unit test without modifying const, but verifies generation works
        assert!(license.contains(':'));
        assert!(license.split(':').count() == 2);
    }
}
