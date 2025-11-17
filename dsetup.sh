#!/bin/bash

echo "[DSetup] Setting up your development environment for Aegisr..."

alias daemon="cargo run --bin aegisr-daemon --"
echo "[DSetup] Alias 'daemon' have been set up for Aegisr."
echo "[DSetup] You can start the Aegisr daemon with 'daemon'."

alias terminal="cargo run --bin aegisr --"
echo "[DSetup] Alias 'terminal' have been set up for Aegisr."
echo "[DSetup] You can start the Aegisr terminal with 'terminal'."

alias repl="cargo run --bin aegisr-repl --"