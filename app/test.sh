#!/bin/sh
# Start de test-Jera uit de code, naast de gewone, en start hem opnieuw zodra de code verandert.
# Gebruik: app/test.sh [map ...]
d="$(cd "$(dirname "$0")" && pwd)"
exec watchexec --restart --watch "$d/src" --watch "$d/fonts" --watch "$d/iconen" --watch "$d/Cargo.toml" \
  -- ~/.cargo/bin/cargo run --manifest-path "$d/Cargo.toml" -- "$@"
