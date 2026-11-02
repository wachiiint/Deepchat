# Wire protocol

Client ↔ server over raw TCP. Both sides depend on one shared definition.

## Frame 
(mean message in Application layer of Kurose model)
```text
┌──────────────────┬──────────────────────┐
│ len: u32 (BE)    │ body: len bytes      │
└──────────────────┴──────────────────────┘
```

| Part | Format |
|------|--------|
| `len` | 4 bytes, big-endian, body length |
| `body` | one message, [postcard](https://docs.rs/postcard)-encoded |

## Client → Server (`ClientMsg`)

| # | Variant | Fields |
|---|---------|--------|
| 0 | `Register` | `name` |
| 1 | `Dm` | `to`, `text` |
| 2 | `CreateGroup` | `group` |
| 3 | `ListGroups` | — |
| 4 | `JoinGroup` | `group` |
| 5 | `GroupMsg` | `group`, `text` |
| 6 | `Pong` | — |
| 7 | `LeaveGroup` | `group` |

## Server → Client (`ServerMsg`)

| # | Variant | Fields |
|---|---------|--------|
| 0 | `Welcome` | `name` |
| 1 | `Users` | `names` (includes the receiver) |
| 2 | `Dm` | `from`, `to`, `text` |
| 3 | `Groups` | list of (`group`, `creator`, `members`) |
| 4 | `GroupMsg` | `group`, `from`, `text` |
| 5 | `Ping` | — |
| 6 | `Error` | `kind` (e.g. name taken, not a member) |

## Behavior

| Topic | Rule |
|-------|------|
| User list | Server pushes `Users` to everyone on each connect/disconnect — no request needed |
| DM delivery | Sent to both sender and receiver |
| DM room id | Sorted pair of names, e.g. `alice` + `bob` → `alice\|bob`; derived by each client |
| Group delivery | Sent to group members only, including the sender |
| Group list | Pushed to every registered client after every group change; `ListGroups` replies only to its requester |
| Liveness | Server sends `Ping` periodically; client replies `Pong`. Server closes a connection that goes silent. Timings: `server-architecture.md` |

## Group lifecycle

- Every registered client can list all groups and their sorted members and creator.
- A creator starts alone. Other users join themselves; membership is never restored on reconnect.
- Only members can send or receive group messages; the sender receives a server echo.
- `Groups` is pushed to every registered client after create, join, leave, or membership cleanup.
- Leaving or disconnecting removes membership. Empty groups are deleted and names can be reused.
- If the creator leaves while others remain, the original creator name stays on the group.
- Desktop history is a cache of received messages for the current connection. Group history is discarded on leave/deletion; it is not replayed on join.

`GroupInfo` now includes `creator` between `group` and `members`. This changes the postcard payload: rebuild the server and client together.
