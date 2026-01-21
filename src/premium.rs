use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, info, warn};

/// Premium tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PremiumTier {
    Free,
    Premium,
}

impl fmt::Display for PremiumTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PremiumTier::Free => write!(f, "Free"),
            PremiumTier::Premium => write!(f, "Premium"),
        }
    }
}

/// Premium features that can be enabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumFeatures {
    pub cloud_sync: bool,
    pub unlimited_history: bool,
    pub cook_profiles: bool,
    pub remote_access: bool,
    pub advanced_analytics: bool,
    pub alerts: bool,
}

impl PremiumFeatures {
    /// Free tier features
    pub fn free() -> Self {
        Self {
            cloud_sync: false,
            unlimited_history: false,
            cook_profiles: false,
            remote_access: false,
            advanced_analytics: false,
            alerts: false,
        }
    }

    /// Premium tier features
    pub fn premium() -> Self {
        Self {
            cloud_sync: true,
            unlimited_history: true,
            cook_profiles: true,
            remote_access: true,
            advanced_analytics: true,
            alerts: true,
        }
    }
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub tier: PremiumTier,
    pub features: PremiumFeatures,
    pub expires_at: Option<DateTime<Utc>>,
    pub issued_at: DateTime<Utc>,
    pub license_key: String,
}

impl License {
    /// Create a free license
    pub fn free() -> Self {
        Self {
            tier: PremiumTier::Free,
            features: PremiumFeatures::free(),
            expires_at: None,
            issued_at: Utc::now(),
            license_key: String::new(),
        }
    }

    /// Check if license is valid (not expired)
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Utc::now() < expiry,
            None => true, // No expiry = lifetime license
        }
    }

    /// Check if license has expired
    pub fn is_expired(&self) -> bool {
        !self.is_valid()
    }

    /// Get days until expiry (None if lifetime)
    pub fn days_until_expiry(&self) -> Option<i64> {
        self.expires_at.map(|expiry| {
            (expiry - Utc::now()).num_days()
        })
    }
}

/// License validator
pub struct LicenseValidator {
    #[allow(dead_code)]
    public_key: Vec<u8>,
}

impl LicenseValidator {
    /// Create a new validator with the public key
    pub fn new() -> Self {
        // In production, embed this at compile time or load from secure location
        // For now, using a placeholder
        Self {
            public_key: Self::default_public_key(),
        }
    }

    /// Validate a license key
    pub fn validate(&self, license_key: &str) -> Result<License> {
        if license_key.is_empty() {
            debug!("Empty license key, using free tier");
            return Ok(License::free());
        }

        // Decode the license key
        let decoded = Self::decode_license(license_key)?;
        
        // Verify signature
        if !self.verify_signature(&decoded) {
            warn!("Invalid license signature");
            return Ok(License::free());
        }

        // Parse license data
        let license = Self::parse_license(&decoded)?;

        // Check if expired
        if license.is_expired() {
            warn!("License expired on {:?}", license.expires_at);
            return Ok(License::free());
        }

        info!("✅ Valid {} license activated", license.tier);
        if let Some(days) = license.days_until_expiry() {
            info!("   License expires in {} days", days);
        } else {
            info!("   Lifetime license");
        }

        Ok(license)
    }

    /// Decode a base64-encoded license key
    fn decode_license(license_key: &str) -> Result<Vec<u8>> {
        // Remove dashes and whitespace
        let cleaned = license_key.replace(['-', ' '], "");
        
        // Decode from base64
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(cleaned.as_bytes())
            .context("Invalid license key format")?;
        
        Ok(decoded)
    }

    /// Verify the signature of a license
    fn verify_signature(&self, _data: &[u8]) -> bool {
        // In production, use RSA or Ed25519 signature verification
        // For now, accept all non-empty keys for development
        // 
        // Real implementation would:
        // 1. Split data into: signature (last 256 bytes) + payload
        // 2. Verify signature against payload using public key
        // 3. Return true only if signature is valid
        true
    }

    /// Parse license data from decoded bytes
    fn parse_license(data: &[u8]) -> Result<License> {
        // Simple format for development:
        // Format: "TIER|EXPIRY|ISSUED"
        // Example: "PREMIUM|2027-01-20T00:00:00Z|2026-01-20T00:00:00Z"
        
        let text = String::from_utf8_lossy(data);
        let parts: Vec<&str> = text.split('|').collect();
        
        if parts.len() < 3 {
            return Ok(License::free());
        }

        let tier = match parts[0] {
            "PREMIUM" => PremiumTier::Premium,
            _ => PremiumTier::Free,
        };

        let expires_at = if parts[1] == "NEVER" {
            None
        } else {
            DateTime::parse_from_rfc3339(parts[1])
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        };

        let issued_at = DateTime::parse_from_rfc3339(parts[2])
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let features = match tier {
            PremiumTier::Premium => PremiumFeatures::premium(),
            PremiumTier::Free => PremiumFeatures::free(),
        };

        Ok(License {
            tier,
            features,
            expires_at,
            issued_at,
            license_key: String::new(),
        })
    }

    /// Get the default public key
    fn default_public_key() -> Vec<u8> {
        // In production, this would be your actual RSA/Ed25519 public key
        // Generated once and embedded in the binary
        vec![0u8; 32]
    }
}

impl Default for LicenseValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a license key (for license generation tool)
pub fn generate_license_key(
    tier: PremiumTier,
    expires_at: Option<DateTime<Utc>>,
) -> Result<String> {
    let issued_at = Utc::now();
    
    let tier_str = match tier {
        PremiumTier::Premium => "PREMIUM",
        PremiumTier::Free => "FREE",
    };
    
    let expiry_str = match expires_at {
        Some(dt) => dt.to_rfc3339(),
        None => "NEVER".to_string(),
    };
    
    let issued_str = issued_at.to_rfc3339();
    
    // Format: TIER|EXPIRY|ISSUED
    let data = format!("{}|{}|{}", tier_str, expiry_str, issued_str);
    
    // In production, sign the data with private key here
    // For now, just encode it
    
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data.as_bytes());
    
    // Format as readable key with dashes
    let formatted = encoded
        .chars()
        .collect::<Vec<char>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("-");
    
    Ok(formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_license() {
        let license = License::free();
        assert_eq!(license.tier, PremiumTier::Free);
        assert!(!license.features.cloud_sync);
        assert!(license.is_valid());
    }

    #[test]
    fn test_license_generation() {
        let key = generate_license_key(PremiumTier::Premium, None).unwrap();
        assert!(!key.is_empty());
        
        let validator = LicenseValidator::new();
        let license = validator.validate(&key).unwrap();
        assert_eq!(license.tier, PremiumTier::Premium);
        assert!(license.is_valid());
    }

    #[test]
    fn test_expired_license() {
        let past = Utc::now() - chrono::Duration::days(30);
        let key = generate_license_key(PremiumTier::Premium, Some(past)).unwrap();
        
        let validator = LicenseValidator::new();
        let license = validator.validate(&key).unwrap();
        // Should fall back to free tier when expired
        assert_eq!(license.tier, PremiumTier::Free);
    }
}
