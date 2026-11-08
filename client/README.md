# Deepchat desktop client

Start the TCP server from the repository root with `cargo run -p server`, then run `pnpm install` and `pnpm tauri dev` in this directory. Connect to `127.0.0.1:7878` locally, or use the server machine's address.

Checks: `pnpm check`, `pnpm build`, and `cargo test --workspace` from the repository root for Rust tests.

## Read / unread

Unread counts and read state live in memory and reset on a new connection. A badge appears next to a DM peer or group when messages arrive while its room is closed, or while the app window is unfocused. The window title includes the total. Your own messages never add unread; bot replies do.

To verify with two desktop clients A and B:

1. Keep B outside A's DM room. Send two DMs from A: B shows badge `2` and `Deepchat (2)` in the title. A has no Seen label yet.
2. Focus B and open A's room. The badge/title clear, and A sees Seen under its newest read message.
3. Leave the room open in B but focus A. Send another message: B gets an unread badge even though the room is open. Refocus B: the badge clears and the new message becomes Seen.
4. In a group with A, B and C, send from A while B and C are elsewhere. When B opens the group, A sees Seen by B. C opening it adds C; non-members get no receipt.
5. Leave/delete a group with unread messages. Its badge is removed from the title total. Reconnecting resets all counts.

`pnpm test` runs the focused client-state regression tests (Node.js 22.12+ with TypeScript stripping); `pnpm check` checks components and types. Rust TCP and client-cache tests are included in `cargo test --workspace`.
