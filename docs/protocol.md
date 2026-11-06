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
| 12 | `Read` | `room`, `upto: u64` |

## Server → Client (`ServerMsg`)

| # | Variant | Fields |
|---|---------|--------|
| 0 | `Ping` | — |
| 1 | `Welcome` | `name` |
| 2 | `Users` | `names` (includes the receiver) |
| 3 | `Dm` | `id: u64`, `from`, `to`, `text` |
| 4 | `Groups` | list of (`group`, `creator`, `members`, `bots`) |
| 5 | `GroupMsg` | `id: u64`, `group`, `from`, `text` |
| 6 | `Error` | `kind` (see below) |
| 7 | `GroupDeleted` | `group` |
| 8 | `Typing` | `room` ([`RoomRef`](#roomref)), `from` |
| 9 | `Read` | `room`, `by`, `upto: u64` |

### `RoomRef`

Names a room without a derived id. Used by `Typing` and `Read` in both directions.

| Variant | Fields | Meaning |
|---------|--------|---------|
| `Dm` | `to` | The **other party** of a private room |
| `Group` | `group` | A group room |

### `ErrorKind` (in `ServerMsg::Error`)

| Kind | Sent when |
|------|-----------|
| `NameTaken` | `Register` with a name that is already connected |
| `NotRegistered` | Any message other than `Register` or `Read` before registering |
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

## Read / unread (F11)

Every accepted `ServerMsg::Dm` and `ServerMsg::GroupMsg` now starts with `id: u64`, assigned by a single server counter starting at 1. Both copies of a DM carry the same id. Human group messages and bot replies use the same sequence. Rejected messages consume no id. IDs reset when the server restarts, along with in-memory client history.

Appended variants (existing variant order is unchanged):

| Direction | Index | Variant | Fields |
|---|---|---|---|
| Client → Server | 12 | `Read` | `room: RoomRef`, `upto: u64` |
| Server → Client | 9 | `Read` | `room: RoomRef`, `by: String`, `upto: u64` |

`RoomRef` is a shared enum: index 0 is `Dm { to: String }`, index 1 is `Group { group: String }`. `to` always names the other party. Thus when B reads a DM from A, B sends `Dm { to: A }` and A receives `Dm { to: B }, by: B`.

A receipt means that `by` has read messages in that room with ids `<= upto`. It is relayed only to the online DM peer, or to every current group member except the reader. The server stores no read state and does not echo receipts. Unregistered readers, self-DMs, offline peers, unknown groups and non-member readers are silently ignored. There is no opt-out of sending receipts.

Unread counts live only in the client. Incoming messages, including bot replies, increment the room badge when that room is closed or the window is unfocused. Own echoes never increment it. Opening a focused room or regaining focus clears its count and sends the latest actually received id if it is newer than that room's last sent receipt. Deleted/left groups drop their counts and read watermark. The title shows the sum, e.g. `Deepchat (3)`.

On the sender, receipts add each reader only once to existing own messages through the specified id; later messages remain unread. DMs show `Seen` under the newest own message that has a reader. Groups show `Seen by A, B`, or `Seen by 4` when more than three readers have read that message. Bot replies have ids and count as unread, but bots do not send receipts.

Desktop IPC represents ids and `upto` as decimal strings, preserving all 64 bits across JavaScript. They remain `u64` on the TCP wire. Rebuild client and server together for the added fields.
