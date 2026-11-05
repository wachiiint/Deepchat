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

1. **Messages** — `ClientMsg` / `ServerMsg` in `protocol/src/messages.rs` (shared `protocol` crate). Wire format: `docs/protocol.md`.
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
| F12 | Part 3 | Typing relay to the other participants |

F4 (chat UI) is client-side. Server delivers room-scoped messages only.

## State

| State | Location |
|--------|----------|
| Online users | In-memory registry |
| Groups / members | In-memory hub maps |
| Bots | Per group: name → personality, max 3 |
| Message history | Last 50 group lines, in memory, only as bot context. Not stored on disk, not replayed to clients |

No generic repository layer in v1. Add a narrow store trait later only if persistence is required.

## Testing

Hub logic is tested in-process, without sockets: `hub/tests.rs` drives `hub::apply` through `hub/testkit.rs` (`Harness` connects fake clients and `drain()` returns the `ServerMsg`s each received). New hub features should ship with tests written the same way.

## Layout

```text
protocol/src/       # shared crate (server + client)
  messages.rs       # ClientMsg, ServerMsg, GroupInfo, RoomRef, ErrorKind
  framer.rs         # length-prefixed frames + postcard encode/decode
server/src/
  main.rs           # bind, accept, spawn
  session.rs        # per-connection reader/writer, ping/pong liveness
  hub.rs            # users, dm, groups, fan-out
  hub/testkit.rs    # cfg(test) harness: fake clients, drains ServerMsgs
  hub/tests.rs      # cfg(test) hub behavior tests
  state.rs          # in-memory users + groups
  ai.rs             # one LLM call: complete(system, prompt)
```

## AI bots (F10)

The LLM is slow (seconds), and the hub must never wait. So the hub only queues the job; a separate task makes the call and reports back as a new event.

```text
"@Pirate hi" → hub: deliver to members, save line, queue AiJob, send Typing from Pirate
hub::run     → spawn task → HTTP → Ollama
task         → Event::AiReply → hub → GroupMsg from Pirate → members only
```

| Item | Value |
|------|-------|
| LLM transport | HTTP to an OpenAI-compatible endpoint. Chat itself stays raw TCP |
| Default backend | Local Ollama, `llama3.2:1b` |
| Config | `DEEPCHAT_AI_URL` (default `http://localhost:11434/v1`), `DEEPCHAT_AI_MODEL` |
| Timeout | 60 s, then the bot posts `(bot unavailable: timeout)` |
| System prompt | `You are <bot>, a bot in a group chat. <personality>` |
| User prompt | The last 50 lines, then the persona again and "Write only <bot>'s short reply to the last message". Small models otherwise just continue the log in another bot's voice |
| Hub → task link | Weak sender, so the hub's own handle does not keep its channel open forever |
| Tests | Check the queued `AiJob`; no LLM, no stub. One `#[ignore]` test calls real Ollama |

**Library spike:** `aisdk` 0.5.2 works with local Ollama through its `openaicompatible` feature. It also needs the `openai` feature, because `openaicompatible` uses that code but does not turn it on.

## Liveness

App-level `Ping`/`Pong` plus a read timeout, both in `session.rs` (`Liveness::DEFAULT`).

| Rule | Value |
|------|-------|
| Server sends `Ping` | every 10 s |
| Client replies `Pong` | the session consumes it; the hub never sees it |
| Server drops a client that sends no frame | after 30 s (any frame counts, not only `Pong`) |

A dropped client goes through the normal disconnect path, so its name is freed and the others see it leave the user list.

The timeout is what detects a dead peer. Alone it would kick healthy idle clients, so `Ping` gives them something to send. The 30 s is 3× the ping interval so one lost `Pong` doesn't kick anyone. WebSocket defines ping/pong but no timings, so these are our own starting values; change them in `Liveness::DEFAULT`.

Clients MUST answer every `Ping`, or they are dropped after 30 s idle.

## v1 out of scope

- SQLite / CRUD repositories
- HTTP or WebSockets for chat
- Mixing framing and hub logic in `main.rs`
