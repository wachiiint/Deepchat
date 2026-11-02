# Deepchat desktop client

Start the TCP server from the repository root with `cargo run -p server`, then run `pnpm install` and `pnpm tauri dev` in this directory. Connect to `127.0.0.1:7878` locally, or use the server machine's address.

Open three desktop clients with different display names to check group chat:

1. A creates `g1`. Every client sees `g1` with creator A and members `[A]`.
2. B joins using Join. All three clients see `[A, B]`; C can see the list but cannot open the room.
3. A and B exchange messages in `g1`. Both receive the server-confirmed messages; C receives none.
4. A leaves. A's room closes, and the group stays with B as its only member.
5. B leaves or disconnects. The group disappears for everyone and its name can be reused.
6. Reconnect A with the same name. A has no inherited group memberships or previous connection history.

Messages, users, and groups come from the TCP server. The desktop client replies to server heartbeats automatically. Disconnect closes the connection and returns to the connection form.

Checks: `pnpm check`, `pnpm build`, and `cargo test --workspace` from the repository root for Rust tests.
