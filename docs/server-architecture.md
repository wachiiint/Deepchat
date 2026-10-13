# Server architecture

Raw TCP between clients and server. Framing/session stay separate from chat application logic. State is in-memory for v1.

## Layers

| Layer | Responsibility | Examples |
|--------|----------------|----------|
| Transport | I/O on the socket | `TcpListener`, accept, read/write, disconnect |
| Session / framing | Message boundaries, liveness | frame read/write, encode/decode, ping/pong, timeouts |
| Application | Chat behavior | register, user list, DM, groups, AI reply |

```text
TCP bytes → Framer → postcard (ClientMsg / ServerMsg) → Session → Hub → Connection registry
```

## Data flow

1. **Messages** — `ClientMsg` / `ServerMsg` in `server/src/protocol.rs`. Wire format: `docs/protocol.md`.
2. **Per-connection task** — read frame → decode → handle session → send app messages to the hub.
3. **Hub** — applies F1–F10 routing; writes outbound `ServerMsg` to recipients.
4. **Registry** — `name → Sender` (session state only).

## Feature → hub

| ID | Course | Concern |
|----|--------|---------|
| F1 | R3 | Unique name on `Register` |
| F2 | R4 | Connected user list / presence events |
| F3 | R5 | Room id for private vs group |
| F5 | R7 | DM fan-out (sender + receiver) |
| F6–F9 | R8–R11 | Group create, list, join, member-only messages |
| F10 | Part 3 | AI reply into a group |

F4 (chat UI) is client-side. Server delivers room-scoped messages only.

## State

| State | Location |
|--------|----------|
| Online users | In-memory registry |
| Groups / members | In-memory hub maps |
| Message history | Not in v1 |
| AI | `AiClient` trait (stub → Ollama / OpenAI) |

No generic repository layer in v1. Add a narrow store trait later only if persistence is required.

## Layout

```text
server/src/
  main.rs           # bind, accept, spawn
  protocol.rs       # ClientMsg, ServerMsg, ErrorKind
  net/
    framer.rs       # length-prefixed frames + postcard encode/decode
  session.rs        # ping/pong, register, conn handle
  hub.rs            # users, dm, groups, fan-out
  state.rs          # in-memory users + groups
  ai.rs             # AiClient + implementations
```

## Open decisions

1. Ping: app-level `Ping`/`Pong`, read timeout, or both

## v1 out of scope

- SQLite / CRUD repositories
- HTTP or WebSockets for chat
- Mixing framing and hub logic in `main.rs`
