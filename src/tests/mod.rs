use crate::validate::validate_fixture;
use crate::fees::{estimate_vbytes, required_fee};
use crate::coinselect::{select_coins, SelectError};
use crate::builder::{compute_locktime, compute_sequence, locktime_type, build};
use crate::types::{Fixture, Utxo, Payment, ChangeTemplate, Policy};

#[test]
fn test_validate_rejects_missing_utxos() {
    let json = serde_json::json!({
        "network": "mainnet",
        "utxos": [],
        "payments": [{"script_pubkey_hex": "00", "script_type": "p2wpkh", "value_sats": 1000}],
        "change": {"script_pubkey_hex": "00", "script_type": "p2wpkh"},
        "fee_rate_sat_vb": 1.0
    });
    let res = validate_fixture(json);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), "MISSING_UTXOS");
}

#[test]
fn test_validate_rejects_invalid_txid() {
    let json = serde_json::json!({
        "network": "mainnet",
        "utxos": [{
            "txid": "too-short",
            "vout": 0,
            "value_sats": 10000,
            "script_pubkey_hex": "00",
            "script_type": "p2wpkh"
        }],
        "payments": [{"script_pubkey_hex": "00", "script_type": "p2wpkh", "value_sats": 1000}],
        "change": {"script_pubkey_hex": "00", "script_type": "p2wpkh"},
        "fee_rate_sat_vb": 1.0
    });
    let res = validate_fixture(json);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), "INVALID_TXID");
}

#[test]
fn test_validate_rejects_zero_fee_rate() {
    let json = serde_json::json!({
        "network": "mainnet",
        "utxos": [{
            "txid": "0000000000000000000000000000000000000000000000000000000000000000",
            "vout": 0,
            "value_sats": 10000,
            "script_pubkey_hex": "00",
            "script_type": "p2wpkh"
        }],
        "payments": [{"script_pubkey_hex": "00", "script_type": "p2wpkh", "value_sats": 1000}],
        "change": {"script_pubkey_hex": "00", "script_type": "p2wpkh"},
        "fee_rate_sat_vb": 0.0
    });
    let res = validate_fixture(json);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().code(), "INVALID_FEE_RATE");
}

#[test]
fn test_estimate_vbytes() {
    let utxo = Utxo {
        txid: "00".repeat(32),
        vout: 0,
        value_sats: 10000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    };
    let outputs = vec!["p2wpkh", "p2wpkh"];
    assert_eq!(estimate_vbytes(&[utxo], &outputs), 141);
}

#[test]
fn test_required_fee() {
    assert_eq!(required_fee(141, 5.0), 705); 
    assert_eq!(required_fee(141, 1.0), 141);
}

#[test]
fn test_select_coins_insufficient_funds() {
    let utxos = vec![Utxo {
        txid: "00".repeat(32),
        vout: 0,
        value_sats: 1000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let payments = vec![Payment {
        value_sats: 2000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let change = ChangeTemplate {
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    };
    let res = select_coins(&utxos, &payments, &change, 1.0, None);
    assert!(matches!(res, Err(SelectError::InsufficientFunds { .. })));
}

#[test]
fn test_select_coins_max_inputs_exceeded() {
    let utxos = vec![
        Utxo { txid: "01".repeat(32), vout: 0, value_sats: 1000, script_pubkey_hex: "00".into(), script_type: "p2wpkh".into(), address: None },
        Utxo { txid: "02".repeat(32), vout: 0, value_sats: 1000, script_pubkey_hex: "00".into(), script_type: "p2wpkh".into(), address: None },
    ];
    let payments = vec![Payment { value_sats: 1500, script_pubkey_hex: "00".into(), script_type: "p2wpkh".into(), address: None }];
    let change = ChangeTemplate { script_pubkey_hex: "00".into(), script_type: "p2wpkh".into(), address: None };
    let policy = Policy { max_inputs: Some(1) };
    let res = select_coins(&utxos, &payments, &change, 1.0, Some(&policy));
    assert!(matches!(res, Err(SelectError::MaxInputsExceeded { .. })));
}

#[test]
fn test_select_coins_send_all() {
    let utxos = vec![Utxo {
        txid: "00".repeat(32),
        vout: 0,
        value_sats: 1200,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let payments = vec![Payment {
        value_sats: 1000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let change = ChangeTemplate {
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    };
    let res = select_coins(&utxos, &payments, &change, 1.0, None).unwrap();
    assert!(res.is_send_all);
    assert!(res.change_sats.is_none());
}

#[test]
fn test_select_coins_with_change() {
    let utxos = vec![Utxo {
        txid: "00".repeat(32),
        vout: 0,
        value_sats: 2000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let payments = vec![Payment {
        value_sats: 1000,
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    }];
    let change = ChangeTemplate {
        script_pubkey_hex: "00".into(),
        script_type: "p2wpkh".into(),
        address: None,
    };
    let res = select_coins(&utxos, &payments, &change, 1.0, None).unwrap();
    assert!(!res.is_send_all);
    assert_eq!(res.change_sats, Some(859));
}

#[test]
fn test_compute_locktime_rbf() {
    let fixture = Fixture {
        network: "mainnet".into(),
        utxos: vec![],
        payments: vec![],
        change: ChangeTemplate { script_pubkey_hex: "".into(), script_type: "".into(), address: None },
        fee_rate_sat_vb: 1.0,
        rbf: Some(true),
        locktime: None,
        current_height: Some(100),
        policy: None,
        extra: Default::default(),
    };
    assert_eq!(compute_locktime(&fixture), 100);
}

#[test]
fn test_compute_locktime_explicit() {
    let fixture = Fixture {
        network: "mainnet".into(),
        utxos: vec![],
        payments: vec![],
        change: ChangeTemplate { script_pubkey_hex: "".into(), script_type: "".into(), address: None },
        fee_rate_sat_vb: 1.0,
        rbf: Some(true),
        locktime: Some(200),
        current_height: Some(100),
        policy: None,
        extra: Default::default(),
    };
    assert_eq!(compute_locktime(&fixture), 200);
}

#[test]
fn test_compute_locktime_none() {
    let fixture = Fixture {
        network: "mainnet".into(),
        utxos: vec![],
        payments: vec![],
        change: ChangeTemplate { script_pubkey_hex: "".into(), script_type: "".into(), address: None },
        fee_rate_sat_vb: 1.0,
        rbf: None,
        locktime: None,
        current_height: Some(100),
        policy: None,
        extra: Default::default(),
    };
    assert_eq!(compute_locktime(&fixture), 0);
}

#[test]
fn test_compute_sequence_rbf() {
    assert_eq!(compute_sequence(true, 0), 0xFFFFFFFD);
}

#[test]
fn test_compute_sequence_locktime() {
    assert_eq!(compute_sequence(false, 100), 0xFFFFFFFE);
}

#[test]
fn test_compute_sequence_none() {
    assert_eq!(compute_sequence(false, 0), 0xFFFFFFFF);
}

#[test]
fn test_locktime_type_none() {
    assert_eq!(locktime_type(0), "none");
}

#[test]
fn test_locktime_type_height() {
    assert_eq!(locktime_type(499_999_999), "block_height");
}

#[test]
fn test_locktime_type_timestamp() {
    assert_eq!(locktime_type(500_000_000), "unix_timestamp");
}

#[test]
fn test_build_balance_check() {
    let fixture = Fixture {
        network: "mainnet".into(),
        utxos: vec![Utxo {
            txid: "00".repeat(32),
            vout: 0,
            value_sats: 5000,
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        }],
        payments: vec![Payment {
            value_sats: 1000,
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        }],
        change: ChangeTemplate {
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        },
        fee_rate_sat_vb: 1.0,
        rbf: None,
        locktime: None,
        current_height: None,
        policy: None,
        extra: Default::default(),
    };
    let report = build(fixture).unwrap();
    let sum_inputs: u64 = report.selected_inputs.iter().map(|i| i.value_sats).sum();
    let sum_outputs: u64 = report.outputs.iter().map(|o| o.value_sats).sum();
    assert_eq!(sum_inputs, sum_outputs + report.fee_sats);
}

#[test]
fn test_build_rbf_warning() {
    let fixture = Fixture {
        network: "mainnet".into(),
        utxos: vec![Utxo {
            txid: "00".repeat(32),
            vout: 0,
            value_sats: 5000,
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        }],
        payments: vec![Payment {
            value_sats: 1000,
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        }],
        change: ChangeTemplate {
            script_pubkey_hex: "00".into(),
            script_type: "p2wpkh".into(),
            address: None,
        },
        fee_rate_sat_vb: 1.0,
        rbf: Some(true),
        locktime: None,
        current_height: None,
        policy: None,
        extra: Default::default(),
    };
    let report = build(fixture).unwrap();
    assert!(report.rbf_signaling);
    assert!(report.warnings.iter().any(|w| w.code == "RBF_SIGNALING"));
}
