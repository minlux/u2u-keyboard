#!/usr/bin/env bash
set -e

ELF="target/thumbv6m-none-eabi/release/u2u-keyboard"
UF2="target/thumbv6m-none-eabi/release/u2u-keyboard.uf2"
MOUNT="/media/$USER/RPI-RP2"

# cargo build --release
# cargo build --release --no-default-features --features layout-qwertz
cargo build --release --no-default-features --features layout-qwerty

elf2uf2-rs "$ELF" "$UF2"

echo ""
read -r -p "Upload $UF2 to $MOUNT? [y/N] " answer
if [[ "$answer" =~ ^[Yy]$ ]]; then
    cp "$UF2" "$MOUNT/"
    echo "Uploaded."
fi
