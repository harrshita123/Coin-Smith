use thiserror::Error;
use crate::types::Fixture;
use serde_json::Value;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{message}")]
    Invalid { code: &'static str, message: String },
}

impl ValidationError {
    pub fn code(&self) -> &str {
        match self {
            Self::Invalid { code, .. } => code,
        }
    }
}

pub fn validate_fixture(raw: Value) -> Result<Fixture, ValidationError> {
    let fixture: Fixture = serde_json::from_value(raw).map_err(|e| ValidationError::Invalid {
        code: "INVALID_FORMAT",
        message: format!("Failed to parse fixture: {}", e),
    })?;

    if fixture.network != "mainnet" && fixture.network != "testnet" {
        return Err(ValidationError::Invalid {
            code: "INVALID_NETWORK",
            message: "Network must be mainnet or testnet".to_string(),
        });
    }

    if fixture.utxos.is_empty() {
        return Err(ValidationError::Invalid {
            code: "MISSING_UTXOS",
            message: "UTXOs cannot be empty".to_string(),
        });
    }

    for (i, utxo) in fixture.utxos.iter().enumerate() {
        if utxo.txid.len() != 64 || !utxo.txid.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::Invalid {
                code: "INVALID_TXID",
                message: format!("Invalid txid at UTXO {}: must be 64 hex chars", i),
            });
        }
        if utxo.value_sats == 0 {
            return Err(ValidationError::Invalid {
                code: "INVALID_UTXO_VALUE",
                message: format!("UTXO {} has zero value", i),
            });
        }
    }

    if fixture.payments.is_empty() {
        return Err(ValidationError::Invalid {
            code: "MISSING_PAYMENTS",
            message: "Payments cannot be empty".to_string(),
        });
    }

    for (i, payment) in fixture.payments.iter().enumerate() {
        if payment.value_sats == 0 {
            return Err(ValidationError::Invalid {
                code: "INVALID_PAYMENT_VALUE",
                message: format!("Payment {} has zero value", i),
            });
        }
    }

    if fixture.change.script_pubkey_hex.is_empty() {
        return Err(ValidationError::Invalid {
            code: "MISSING_CHANGE_SCRIPT",
            message: "Change script_pubkey_hex is required".to_string(),
        });
    }

    if fixture.fee_rate_sat_vb <= 0.0 {
        return Err(ValidationError::Invalid {
            code: "INVALID_FEE_RATE",
            message: "Fee rate must be greater than zero".to_string(),
        });
    }

    if let Some(policy) = &fixture.policy {
        if let Some(max_inputs) = policy.max_inputs {
            if max_inputs < 1 {
                return Err(ValidationError::Invalid {
                    code: "INVALID_POLICY",
                    message: "max_inputs must be at least 1".to_string(),
                });
            }
        }
    }

    Ok(fixture)
}
