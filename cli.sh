#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

if [ $# -lt 1 ]; then
    echo "Usage: ./cli.sh <fixture.json>" >&2
    exit 1
fi

cargo build --release 2>/dev/null
./target/release/coin-smith "$1"
