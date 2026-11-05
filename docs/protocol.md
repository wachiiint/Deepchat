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
| 0 | `Pong` | — |
| 1 | `Register` | `name` |
| 2 | `Dm` | `to`, `text` |
| 3 | `CreateGroup` | `group` |
| 4 | `ListGroups` | — |
| 5 | `JoinGroup` | `group` |
| 6 | `GroupMsg` | `group`, `text` |
| 7 | `LeaveGroup` | `group` |
| 8 | `DeleteGroup` | `group` |
| 9 | `Typing` | `room` ([`RoomRef`](#roomref)) |
| 10 | `CreateBot` | `group`, `name`, `personality` |
| 11 | `RemoveBot` | `group`, `name` |

## Server → Client (`ServerMsg`)

| # | Variant | Fields |
|---|---------|--------|
| 0 | `Ping` | — |
| 1 | `Welcome` | `name` |
| 2 | `Users` | `names` (includes the receiver) |
| 3 | `Dm` | `from`, `to`, `text` |
| 4 | `Groups` | list of (`group`, `creator`, `members`, `bots`) |
| 5 | `GroupMsg` | `group`, `from`, `text` |
| 6 | `Error` | `kind` (see below) |
| 7 | `GroupDeleted` | `group` |
| 8 | `Typing` | `room` ([`RoomRef`](#roomref)), `from` |

### `RoomRef`

Names a room without a derived id. Used by `Typing` in both directions.

| Variant | Fields | Meaning |
|---------|--------|---------|
| `Dm` | `to` | The **other party** of a private room |
| `Group` | `group` | A group room |

### `ErrorKind` (in `ServerMsg::Error`)

| Kind | Sent when |
|------|-----------|
| `NameTaken` | `Register` with a name that is already connected |
| `NotRegistered` | Any message other than `Register` before registering |
| `UnknownUser` | `Dm` to a user who is not connected |
| `GroupExists` | `CreateGroup` with a name that is already in use |
| `UnknownGroup` | `JoinGroup`, `LeaveGroup`, `DeleteGroup` or `GroupMsg` for a group that does not exist |
| `NotMember` | `LeaveGroup` or `GroupMsg` by someone who is not in the group |
| `NotCreator` | `DeleteGroup`, `CreateBot` or `RemoveBot` by someone who did not create the group |
| `BotLimit` | `CreateBot` when the group already has 3 bots, or the personality is over 500 characters |
| `BotNameTaken` | `CreateBot` with a name already used by a bot in that group or by a connected user |

## Behavior

| Topic | Rule |
|-------|------|
| User list | Server pushes `Users` to everyone on each connect/disconnect — no request needed |
| DM delivery | Sent to both sender and receiver |
| Group delivery | Sent to group members only, including the sender |
| Group list | Pushed to every registered client after every group change; `ListGroups` replies only to its requester |
| Group membership | Creator starts as the only member; others join with `JoinGroup`. `LeaveGroup` or disconnect removes membership (never restored on reconnect). A group with no members is deleted and its name can be reused. `creator` never changes, even if the creator leaves |
| Repeated `JoinGroup` | No-op: no reply and no `Groups` push |
| Group delete | Only the creator may send `DeleteGroup`, even after leaving the group. Members at that moment, plus the creator, get `GroupDeleted`, then everyone gets the
new `Groups` |
| Typing | `Typing` is a transient signal: relayed to the other participants, never echoed to the sender, never stored, no "stopped typing" message (the receiving client expires the indicator). DM: relayed to the target if connected. Group: relayed to every member except the sender, only if the sender is a member. Signals never produce `Error`: an offline target, unknown group, non-member sender and typing to yourself are dropped silently. The only error is `NotRegistered` before `Register`, as for every message |
| Bots (F10) | Only the group creator adds or removes bots. Bot names go out in `Groups`; the personality never leaves the server. A `GroupMsg` starting with `@<bot>` (exact case) makes that bot answer: members get `Typing` from the bot, then its reply as a normal `GroupMsg` with `from` = bot name. Bot replies never trigger other bots. If the AI fails, the reply text is `(bot unavailable: …)`. Bots disappear with their group. Removing an unknown bot is a no-op |
| Liveness | Server sends `Ping` periodically; client replies `Pong`. Server closes a connection that goes silent. Timings: `server-architecture.md` |
