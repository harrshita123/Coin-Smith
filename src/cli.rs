use std::{fs, path::Path};
use crate::{validate::validate_fixture, builder::build};

pub fn run(fixture_path: &str) {
    let fixture_name = Path::new(fixture_path)
        .file_stem().expect("valid path").to_str().expect("utf8").to_owned();
    
    fs::create_dir_all("out").unwrap();
    let out_path = format!("out/{}.json", fixture_name);

    use crate::types::{Report, ErrorReport, ErrorDetail};

    let (exit_code, output_json) = match (|| -> anyhow::Result<Report> {
        let content = fs::read_to_string(fixture_path)?;
        let raw: serde_json::Value = serde_json::from_str(&content)?;
        
        match validate_fixture(raw)
            .map_err(|e| (e.code().to_string(), e.to_string()))
            .and_then(|f| build(f).map_err(|e| ("BUILD_ERROR".to_string(), e.to_string())))
        {
            Ok(report) => Ok(Report::Ok(report)),
            Err((code, msg)) => Ok(Report::Err(ErrorReport {
                ok: false,
                error: ErrorDetail { code, message: msg },
            })),
        }
    })() {
        Ok(Report::Ok(r)) => (0, serde_json::to_string_pretty(&Report::Ok(r)).unwrap()),
        Ok(Report::Err(e)) => (1, serde_json::to_string_pretty(&Report::Err(e)).unwrap()),
        Err(e) => (1, serde_json::to_string_pretty(&Report::Err(ErrorReport {
            ok: false,
            error: ErrorDetail {
                code: "SYSTEM_ERROR".into(),
                message: e.to_string(),
            },
        })).unwrap()),
    };

    fs::write(&out_path, output_json).unwrap();
    std::process::exit(exit_code);
}
