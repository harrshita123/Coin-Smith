mod cli;
mod server;
mod builder;
mod coinselect;
mod fees;
mod validate;
mod psbt;
mod types;
#[cfg(test)]
mod tests;

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
