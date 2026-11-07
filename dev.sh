#!/usr/bin/env bash

# Server + 2 clients for local testing. Ctrl+C stops all.
trap 'kill 0' EXIT
cargo run -p server &
cd client 
pnpm tauri dev &
sleep 3
pnpm tauri dev &
wait
