use crate::types::Utxo;

pub fn input_vbytes(script_type: &str) -> u64 {
    match script_type {
        "p2wpkh" => 68,
        "p2wsh" => 105,
        "p2pkh" => 148,
        "p2sh" | "p2sh-p2wpkh" => 91,
        "p2tr" => 58,
        _ => 148,
    }
}

pub fn output_vbytes(script_type: &str) -> u64 {
    match script_type {
        "p2wpkh" => 31,
        "p2wsh" => 43,
        "p2pkh" => 34,
        "p2sh" => 32,
        "p2tr" => 43,
        _ => 34,
    }
}

fn varint_len(n: usize) -> u64 {
    if n < 253 { 1 }
    else if n <= 0xFFFF { 3 }
    else { 5 }
}

pub fn estimate_vbytes(inputs: &[Utxo], output_types: &[&str]) -> u64 {
    let base_header: u64 = 8; 
    let counts: u64 = varint_len(inputs.len()) + varint_len(output_types.len());
    
    let is_any_segwit = inputs.iter().any(|u| is_segwit(&u.script_type));
    let segwit_overhead = if is_any_segwit { 1 } else { 0 }; 
    
    let input_total: u64 = inputs.iter().map(|u| input_vbytes(&u.script_type)).sum();
    let output_total: u64 = output_types.iter().map(|t| output_vbytes(t)).sum();
    
    base_header + counts + segwit_overhead + input_total + output_total
}

pub fn required_fee(vbytes: u64, fee_rate: f64) -> u64 {
    (vbytes as f64 * fee_rate).ceil() as u64
}

fn is_segwit(script_type: &str) -> bool {
    matches!(script_type, "p2wpkh" | "p2wsh" | "p2tr" | "p2sh-p2wpkh")
}
