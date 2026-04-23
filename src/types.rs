use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value_sats: u64,
    pub script_pubkey_hex: String,
    pub script_type: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub address: Option<String>,
    pub script_pubkey_hex: String,
    pub script_type: String,
    pub value_sats: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTemplate {
    pub address: Option<String>,
    pub script_pubkey_hex: String,
    pub script_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Policy {
    pub max_inputs: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub network: String,
    pub utxos: Vec<Utxo>,
    pub payments: Vec<Payment>,
    pub change: ChangeTemplate,
    pub fee_rate_sat_vb: f64,
    pub rbf: Option<bool>,
    pub locktime: Option<u32>,
    pub current_height: Option<u32>,
    pub policy: Option<Policy>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEntry {
    pub n: usize,
    pub value_sats: u64,
    pub script_pubkey_hex: String,
    pub script_type: String,
    pub address: Option<String>,
    pub is_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocktimeType {
    None,
    BlockHeight,
    UnixTimestamp,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Report {
    Ok(BuildReport),
    Err(ErrorReport),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildReport {
    pub ok: bool,  
    pub network: String,
    pub strategy: String,
    pub selected_inputs: Vec<Utxo>,
    pub outputs: Vec<OutputEntry>,
    pub change_index: Option<usize>,
    pub fee_sats: u64,
    pub fee_rate_sat_vb: f64,
    pub vbytes: u64,
    pub rbf_signaling: bool,
    pub locktime: u32,
    pub locktime_type: String,
    pub psbt_base64: String,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReport {
    pub ok: bool,  
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
