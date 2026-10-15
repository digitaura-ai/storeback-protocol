//! Storeback Protocol RFC-0402 Cleanroom Engine
//!
//! Provides strongly-typed zero-float financial arithmetic (Drops, UsdcMicro),
//! RFC-0402 HTTP 402 challenge/proof serialization and deserialization,
//! and settlement types for autonomous machine-to-machine knowledge vending.
//!
//! Cleanroom IP Shield: This crate is strictly dual-licensed under Apache 2.0
//! and contains zero proprietary Digitaura General Ledger or Vault secrets.

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Error types for RFC-0402 protocol processing
#[derive(Debug, Error, PartialEq, Eq)]
pub enum X402Error {
    #[error("Invalid or missing RFC-0402 header: {0}")]
    InvalidHeader(String),

    #[error("Malformed payment proof: {0}")]
    MalformedProof(String),

    #[error("Financial arithmetic overflow")]
    ArithmeticOverflow,

    #[error("Invalid base64 payload: {0}")]
    Base64Error(String),

    #[error("JSON serialization error: {0}")]
    JsonError(String),
}

/// Strongly typed XRP Drops (1 XRP = 1,000,000 Drops).
/// Guaranteed zero floating-point imprecision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Drops(pub u64);

impl Drops {
    pub const DROPS_PER_XRP: u64 = 1_000_000;
    pub const ZERO: Self = Drops(0);

    pub fn from_drops(drops: u64) -> Self {
        Drops(drops)
    }

    pub fn from_xrp_f64(xrp: f64) -> Result<Self, X402Error> {
        if xrp < 0.0 || xrp.is_nan() || xrp.is_infinite() {
            return Err(X402Error::InvalidHeader("Invalid XRP decimal".into()));
        }
        let drops = (xrp * Self::DROPS_PER_XRP as f64).round();
        if drops > u64::MAX as f64 {
            return Err(X402Error::ArithmeticOverflow);
        }
        Ok(Drops(drops as u64))
    }

    pub fn to_xrp_f64(self) -> f64 {
        self.0 as f64 / Self::DROPS_PER_XRP as f64
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, X402Error> {
        self.0
            .checked_add(other.0)
            .map(Drops)
            .ok_or(X402Error::ArithmeticOverflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, X402Error> {
        self.0
            .checked_sub(other.0)
            .map(Drops)
            .ok_or(X402Error::ArithmeticOverflow)
    }
}

impl fmt::Display for Drops {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} drops ({:.6} XRP)", self.0, self.to_xrp_f64())
    }
}

/// Strongly typed USDC Micros (1 USDC = 1,000,000 Micros, 6 decimals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UsdcMicro(pub u64);

impl UsdcMicro {
    pub const MICROS_PER_USDC: u64 = 1_000_000;
    pub const ZERO: Self = UsdcMicro(0);

    pub fn from_micros(micros: u64) -> Self {
        UsdcMicro(micros)
    }

    pub fn from_usdc_f64(usdc: f64) -> Result<Self, X402Error> {
        if usdc < 0.0 || usdc.is_nan() || usdc.is_infinite() {
            return Err(X402Error::InvalidHeader("Invalid USDC decimal".into()));
        }
        let micros = (usdc * Self::MICROS_PER_USDC as f64).round();
        if micros > u64::MAX as f64 {
            return Err(X402Error::ArithmeticOverflow);
        }
        Ok(UsdcMicro(micros as u64))
    }

    pub fn to_usdc_f64(self) -> f64 {
        self.0 as f64 / Self::MICROS_PER_USDC as f64
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, X402Error> {
        self.0
            .checked_add(other.0)
            .map(UsdcMicro)
            .ok_or(X402Error::ArithmeticOverflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, X402Error> {
        self.0
            .checked_sub(other.0)
            .map(UsdcMicro)
            .ok_or(X402Error::ArithmeticOverflow)
    }
}

impl fmt::Display for UsdcMicro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} micros (${:.2} USDC)", self.0, self.to_usdc_f64())
    }
}

/// Supported payment chains for RFC-0402 settlement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentChain {
    Xrpl,
    Base,
}

impl fmt::Display for PaymentChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentChain::Xrpl => write!(f, "xrpl"),
            PaymentChain::Base => write!(f, "base"),
        }
    }
}

/// RFC-0402 HTTP 402 Challenge Descriptor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct X402Challenge {
    pub version: String,
    pub realm: String,
    pub slug: String,
    pub price_drops: Drops,
    pub price_usdc_micros: UsdcMicro,
    pub recipient_xrpl: String,
    pub recipient_base_usdc: String,
    pub nonce: String,
    pub expires_at: u64,
    pub digest: Option<String>,
}

impl X402Challenge {
    /// Serializes challenge into standard HTTP response headers
    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            (
                "WWW-Authenticate".into(),
                format!("x402 realm=\"{}\"", self.realm),
            ),
            ("X-402-Version".into(), self.version.clone()),
            ("X-402-Slug".into(), self.slug.clone()),
            ("X-402-Price-Drops".into(), self.price_drops.as_u64().to_string()),
            (
                "X-402-Price-XRP".into(),
                format!("{:.6}", self.price_drops.to_xrp_f64()),
            ),
            (
                "X-402-Price-USDC-Micros".into(),
                self.price_usdc_micros.as_u64().to_string(),
            ),
            (
                "X-402-Price-USDC".into(),
                format!("{:.2}", self.price_usdc_micros.to_usdc_f64()),
            ),
            (
                "X-402-Recipient-XRPL".into(),
                self.recipient_xrpl.clone(),
            ),
            (
                "X-402-Recipient-Base-USDC".into(),
                self.recipient_base_usdc.clone(),
            ),
            ("X-402-Nonce".into(), self.nonce.clone()),
            ("X-402-Expires".into(), self.expires_at.to_string()),
        ];
        if let Some(ref d) = self.digest {
            headers.push(("X-402-Digest".into(), d.clone()));
        }
        headers
    }

    /// Parses challenge headers from a raw HTTP response header map
    pub fn from_headers(headers: &[(String, String)]) -> Result<Self, X402Error> {
        let mut map: HashMap<String, String> = HashMap::new();
        for (k, v) in headers {
            map.insert(k.to_lowercase(), v.clone());
        }

        let version = map
            .get("x-402-version")
            .cloned()
            .unwrap_or_else(|| "1.0".into());
        let slug = map
            .get("x-402-slug")
            .cloned()
            .ok_or_else(|| X402Error::InvalidHeader("Missing X-402-Slug".into()))?;

        let drops_val = map
            .get("x-402-price-drops")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let micros_val = map
            .get("x-402-price-usdc-micros")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let recipient_xrpl = map
            .get("x-402-recipient-xrpl")
            .cloned()
            .unwrap_or_default();

        let recipient_base_usdc = map
            .get("x-402-recipient-base-usdc")
            .cloned()
            .unwrap_or_default();

        let nonce = map
            .get("x-402-nonce")
            .cloned()
            .ok_or_else(|| X402Error::InvalidHeader("Missing X-402-Nonce".into()))?;

        let expires_at = map
            .get("x-402-expires")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let digest = map.get("x-402-digest").cloned();

        Ok(X402Challenge {
            version,
            realm: "Storeback Sovereign Data Vending".into(),
            slug,
            price_drops: Drops(drops_val),
            price_usdc_micros: UsdcMicro(micros_val),
            recipient_xrpl,
            recipient_base_usdc,
            nonce,
            expires_at,
            digest,
        })
    }
}

/// Standard Payment Proof delivered by autonomous buyers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct X402Proof {
    pub chain: PaymentChain,
    pub tx_hash: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_drops: Option<Drops>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_usdc_micros: Option<UsdcMicro>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer_address: Option<String>,
}

impl X402Proof {
    /// Formats proof as standard Authorization header:
    /// `Authorization: x402 tx_hash=<hex> nonce=<nonce> chain=<chain>`
    pub fn to_authorization_header(&self) -> String {
        format!(
            "x402 tx_hash={} nonce={} chain={}",
            self.tx_hash, self.nonce, self.chain
        )
    }

    /// Parses key-value pairs from `Authorization: x402 ...`
    pub fn from_authorization_header(header: &str) -> Result<Self, X402Error> {
        let trimmed = header.trim();
        if !trimmed.to_lowercase().starts_with("x402") {
            return Err(X402Error::MalformedProof(
                "Header does not start with x402".into(),
            ));
        }

        let params = trimmed[4..].trim();
        let mut map: HashMap<String, String> = HashMap::new();

        // Support key=value separated by spaces or commas
        for token in params.split(|c| c == ' ' || c == ',') {
            let part = token.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((k, v)) = part.split_once('=') {
                let clean_val = v.trim_matches(|c| c == '"' || c == '\'');
                map.insert(k.trim().to_lowercase(), clean_val.to_string());
            }
        }

        let tx_hash = map
            .get("tx_hash")
            .or_else(|| map.get("txid"))
            .or_else(|| map.get("hash"))
            .cloned()
            .ok_or_else(|| X402Error::MalformedProof("Missing tx_hash in authorization".into()))?;

        let nonce = map
            .get("nonce")
            .cloned()
            .ok_or_else(|| X402Error::MalformedProof("Missing nonce in authorization".into()))?;

        let chain_str = map
            .get("chain")
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                if tx_hash.starts_with("0x") {
                    "base"
                } else {
                    "xrpl"
                }
            });

        let chain = match chain_str.to_lowercase().as_str() {
            "base" | "ethereum" | "evm" => PaymentChain::Base,
            _ => PaymentChain::Xrpl,
        };

        Ok(X402Proof {
            chain,
            tx_hash,
            nonce,
            amount_drops: None,
            amount_usdc_micros: None,
            payer_address: map.get("payer").cloned(),
        })
    }

    /// Formats proof as base64-encoded JSON for `X-Payment` header
    pub fn to_x_payment_header(&self) -> Result<String, X402Error> {
        let json_str = serde_json::to_string(self)
            .map_err(|e| X402Error::JsonError(e.to_string()))?;
        Ok(BASE64_STANDARD.encode(json_str.as_bytes()))
    }

    /// Parses proof from base64 JSON `X-Payment` header
    pub fn from_x_payment_header(header: &str) -> Result<Self, X402Error> {
        let trimmed = header.trim();
        let bytes = BASE64_STANDARD
            .decode(trimmed)
            .map_err(|e| X402Error::Base64Error(e.to_string()))?;
        let proof: X402Proof = serde_json::from_slice(&bytes)
            .map_err(|e| X402Error::JsonError(e.to_string()))?;
        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_arithmetic() {
        let d1 = Drops::from_xrp_f64(0.5).unwrap();
        assert_eq!(d1.as_u64(), 500_000);
        assert_eq!(d1.to_xrp_f64(), 0.5);

        let d2 = Drops::from_drops(250_000);
        let sum = d1.checked_add(d2).unwrap();
        assert_eq!(sum.as_u64(), 750_000);
        assert_eq!(sum.to_xrp_f64(), 0.75);

        let diff = sum.checked_sub(d1).unwrap();
        assert_eq!(diff.as_u64(), 250_000);

        assert_eq!(
            d2.checked_sub(d1),
            Err(X402Error::ArithmeticOverflow)
        );
    }

    #[test]
    fn test_usdc_micro_arithmetic() {
        let u1 = UsdcMicro::from_usdc_f64(0.10).unwrap();
        assert_eq!(u1.as_u64(), 100_000);
        assert_eq!(u1.to_usdc_f64(), 0.10);

        let u2 = UsdcMicro::from_micros(50_000);
        let sum = u1.checked_add(u2).unwrap();
        assert_eq!(sum.as_u64(), 150_000);
        assert_eq!(sum.to_usdc_f64(), 0.15);
    }

    #[test]
    fn test_challenge_header_roundtrip() {
        let challenge = X402Challenge {
            version: "1.0".into(),
            realm: "Storeback Sovereign Data Vending".into(),
            slug: "defense-metals-2026".into(),
            price_drops: Drops(500_000),
            price_usdc_micros: UsdcMicro(100_000),
            recipient_xrpl: "rTestOperator123456789".into(),
            recipient_base_usdc: "0x1234567890abcdef1234567890abcdef12345678".into(),
            nonce: "test_nonce_abc123".into(),
            expires_at: 1760000000,
            digest: Some("sha256_mock_digest".into()),
        };

        let headers = challenge.to_headers();
        let parsed = X402Challenge::from_headers(&headers).unwrap();

        assert_eq!(parsed.slug, challenge.slug);
        assert_eq!(parsed.price_drops, challenge.price_drops);
        assert_eq!(parsed.price_usdc_micros, challenge.price_usdc_micros);
        assert_eq!(parsed.recipient_xrpl, challenge.recipient_xrpl);
        assert_eq!(parsed.nonce, challenge.nonce);
        assert_eq!(parsed.expires_at, challenge.expires_at);
        assert_eq!(parsed.digest, challenge.digest);
    }

    #[test]
    fn test_authorization_header_parsing() {
        let proof = X402Proof {
            chain: PaymentChain::Xrpl,
            tx_hash: "ABCDEF0123456789".into(),
            nonce: "nonce_4920".into(),
            amount_drops: Some(Drops(500_000)),
            amount_usdc_micros: None,
            payer_address: None,
        };

        let auth_hdr = proof.to_authorization_header();
        assert_eq!(auth_hdr, "x402 tx_hash=ABCDEF0123456789 nonce=nonce_4920 chain=xrpl");

        let parsed = X402Proof::from_authorization_header(&auth_hdr).unwrap();
        assert_eq!(parsed.tx_hash, "ABCDEF0123456789");
        assert_eq!(parsed.nonce, "nonce_4920");
        assert_eq!(parsed.chain, PaymentChain::Xrpl);
    }

    #[test]
    fn test_x_payment_base64_roundtrip() {
        let proof = X402Proof {
            chain: PaymentChain::Base,
            tx_hash: "0xabcdef123456".into(),
            nonce: "nonce_base_99".into(),
            amount_drops: None,
            amount_usdc_micros: Some(UsdcMicro(100_000)),
            payer_address: Some("0xpayer123".into()),
        };

        let header = proof.to_x_payment_header().unwrap();
        let parsed = X402Proof::from_x_payment_header(&header).unwrap();
        assert_eq!(parsed, proof);
    }

    #[test]
    fn test_malformed_proof_rejection() {
        let res = X402Proof::from_authorization_header("Bearer some_jwt_token");
        assert!(res.is_err());

        let res2 = X402Proof::from_authorization_header("x402 missing_fields");
        assert!(res2.is_err());
    }
}
