use coin_smith::{cli, server};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("serve") => server::run().await,
        Some(fixture_path) => cli::run(fixture_path),
        None => {
            eprintln!("Usage: coin-smith <fixture.json> | coin-smith serve");
            std::process::exit(1);
        }
    }
}
