use crate::types::{Utxo, Payment, ChangeTemplate, Policy};
use crate::fees::{estimate_vbytes, required_fee};
use thiserror::Error;

const DUST_THRESHOLD: u64 = 546;

pub struct SelectResult {
    pub selected: Vec<Utxo>,
    pub change_sats: Option<u64>,
    pub fee_sats: u64,
    pub vbytes: u64,
    pub is_send_all: bool,
}

#[derive(Debug, Error)]
pub enum SelectError {
    #[error("Insufficient funds: need {need} sats, have {have}")]
    InsufficientFunds { need: u64, have: u64 },
    #[error("Cannot cover payments within max_inputs={limit} constraint")]
    MaxInputsExceeded { limit: usize },
}

pub fn select_coins(
    utxos: &[Utxo],
    payments: &[Payment],
    change: &ChangeTemplate,
    fee_rate: f64,
    policy: Option<&Policy>,
) -> Result<SelectResult, SelectError> {
    let max_inputs = policy.and_then(|p| p.max_inputs).unwrap_or(usize::MAX);
    let payment_sum: u64 = payments.iter().map(|p| p.value_sats).sum();
    
    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| b.value_sats.cmp(&a.value_sats));
    
    let mut selected = Vec::new();
    let mut input_sum: u64 = 0;
    
    for (i, utxo) in sorted.iter().take(max_inputs).enumerate() {
        selected.push(utxo.clone());
        input_sum += utxo.value_sats;
        
        let output_types_no_change: Vec<&str> = payments.iter().map(|p| p.script_type.as_str()).collect();
        let vb_no_change = estimate_vbytes(&selected, &output_types_no_change);
        let fee_no_change = required_fee(vb_no_change, fee_rate);
        
        if input_sum >= payment_sum + fee_no_change {
            let mut output_types_with_change = output_types_no_change.clone();
            output_types_with_change.push(&change.script_type);
            let vb_with_change = estimate_vbytes(&selected, &output_types_with_change);
            let fee_with_change = required_fee(vb_with_change, fee_rate);
            
            if input_sum >= payment_sum + fee_with_change {
                let change_sats = input_sum - payment_sum - fee_with_change;
                if change_sats >= DUST_THRESHOLD {
                    return Ok(SelectResult {
                        selected,
                        change_sats: Some(change_sats),
                        fee_sats: fee_with_change,
                        vbytes: vb_with_change,
                        is_send_all: false,
                    });
                }
            }
            
            if i + 1 == sorted.len() || i + 1 == max_inputs {
                let actual_fee = input_sum - payment_sum;
                return Ok(SelectResult {
                    selected,
                    change_sats: None,
                    fee_sats: actual_fee,
                    vbytes: vb_no_change,
                    is_send_all: true,
                });
            }
        }
    }
    
    if selected.len() >= max_inputs && input_sum < payment_sum {
        return Err(SelectError::MaxInputsExceeded { limit: max_inputs });
    }
    
    Err(SelectError::InsufficientFunds { need: payment_sum, have: input_sum })
}
