use crate::types::*;
use crate::coinselect::select_coins;
use crate::psbt::build_psbt;

pub fn compute_locktime(fixture: &Fixture) -> u32 {
    if let Some(lt) = fixture.locktime { return lt; }
    if fixture.rbf == Some(true) {
        if let Some(h) = fixture.current_height { return h; }
    }
    0
}

pub fn compute_sequence(rbf: bool, n_locktime: u32) -> u32 {
    if rbf { return 0xFFFFFFFD; }
    if n_locktime != 0 { return 0xFFFFFFFE; }
    0xFFFFFFFF
}

pub fn locktime_type(n: u32) -> &'static str {
    if n == 0 { "none" }
    else if n < 500_000_000 { "block_height" }
    else { "unix_timestamp" }
}

pub fn build(fixture: Fixture) -> Result<BuildReport, anyhow::Error> {
    let result = select_coins(
        &fixture.utxos,
        &fixture.payments,
        &fixture.change,
        fixture.fee_rate_sat_vb,
        fixture.policy.as_ref(),
    ).map_err(|e| anyhow::anyhow!("{}", e))?;

    let n_locktime = compute_locktime(&fixture);
    let rbf = fixture.rbf.unwrap_or(false);
    let n_sequence = compute_sequence(rbf, n_locktime);
    let rbf_signaling = n_sequence <= 0xFFFFFFFD;

    let mut outputs: Vec<OutputEntry> = fixture.payments.iter().enumerate().map(|(i, p)| {
        OutputEntry {
            n: i,
            value_sats: p.value_sats,
            script_pubkey_hex: p.script_pubkey_hex.clone(),
            script_type: p.script_type.clone(),
            address: p.address.clone(),
            is_change: false,
        }
    }).collect();

    let change_index = if let Some(change_sats) = result.change_sats {
        let idx = outputs.len();
        outputs.push(OutputEntry {
            n: idx,
            value_sats: change_sats,
            script_pubkey_hex: fixture.change.script_pubkey_hex.clone(),
            script_type: fixture.change.script_type.clone(),
            address: fixture.change.address.clone(),
            is_change: true,
        });
        Some(idx)
    } else {
        None
    };

    let psbt_base64 = build_psbt(&result.selected, &outputs, n_locktime, n_sequence)?;

    let actual_fee_rate = if result.vbytes > 0 { result.fee_sats as f64 / result.vbytes as f64 } else { 0.0 };

    let mut warnings = Vec::new();
    if result.fee_sats > 1_000_000 || actual_fee_rate > 200.0 {
        warnings.push(Warning { code: "HIGH_FEE".into(), message: None });
    }
    if result.is_send_all {
        warnings.push(Warning { code: "SEND_ALL".into(), message: None });
    }
    if rbf_signaling {
        warnings.push(Warning { code: "RBF_SIGNALING".into(), message: None });
    }
    if let Some(ci) = change_index {
        if outputs[ci].value_sats < 546 {
            warnings.push(Warning { code: "DUST_CHANGE".into(), message: None });
        }
    }

    let actual_fee_rate = result.fee_sats as f64 / result.vbytes as f64;

    Ok(BuildReport {
        ok: true,
        network: fixture.network,
        strategy: "greedy_bnb".into(),
        selected_inputs: result.selected,
        outputs,
        change_index,
        fee_sats: result.fee_sats,
        fee_rate_sat_vb: (actual_fee_rate * 100.0).round() / 100.0,
        vbytes: result.vbytes,
        rbf_signaling,
        locktime: n_locktime,
        locktime_type: locktime_type(n_locktime).into(),
        psbt_base64,
        warnings,
    })
}
