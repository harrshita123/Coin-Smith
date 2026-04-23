#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release 2>/dev/null
PORT="${PORT:-3000}" ./target/release/coin-smith serve
