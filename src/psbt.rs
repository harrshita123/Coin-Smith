use bitcoin::{
    psbt::Psbt,
    transaction::{Transaction, TxIn, TxOut, Version},
    OutPoint, Txid, ScriptBuf, Sequence, Amount,
    absolute,
};
use std::str::FromStr;
use crate::types::{Utxo, OutputEntry};

pub fn build_psbt(
    selected: &[Utxo],
    outputs: &[OutputEntry],
    n_locktime: u32,
    n_sequence: u32,
) -> Result<String, anyhow::Error> {
    let inputs: Vec<TxIn> = selected.iter().map(|u| {
        TxIn {
            previous_output: OutPoint {
                txid: Txid::from_str(&u.txid).expect("valid txid"),
                vout: u.vout,
            },
            sequence: Sequence(n_sequence),
            ..Default::default()
        }
    }).collect();

    let tx_outputs: Vec<TxOut> = outputs.iter().map(|o| {
        TxOut {
            value: Amount::from_sat(o.value_sats),
            script_pubkey: ScriptBuf::from_hex(&o.script_pubkey_hex).expect("valid script"),
        }
    }).collect();

    let locktime = if n_locktime < 500_000_000 {
        absolute::LockTime::from(absolute::Height::from_consensus(n_locktime).unwrap())
    } else {
        absolute::LockTime::from(absolute::Time::from_consensus(n_locktime).unwrap())
    };
    let locktime = if n_locktime == 0 { absolute::LockTime::ZERO } else { locktime };

    let tx = Transaction {
        version: Version::TWO,
        lock_time: locktime,
        input: inputs,
        output: tx_outputs,
    };

    let mut psbt = Psbt::from_unsigned_tx(tx)?;

    for (i, utxo) in selected.iter().enumerate() {
        psbt.inputs[i].witness_utxo = Some(TxOut {
            value: Amount::from_sat(utxo.value_sats),
            script_pubkey: ScriptBuf::from_hex(&utxo.script_pubkey_hex)?,
        });
    }

    use bitcoin::base64::{Engine, engine::general_purpose::STANDARD};
    Ok(STANDARD.encode(psbt.serialize()))
}
