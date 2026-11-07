# Deepchat

A multi-client chat application built with **TCP socket programming in Rust** and a **Tauri** desktop GUI (Svelte + TypeScript frontend, Rust client backend).

## Overview

Clients talk to a Rust TCP server; the desktop UI is Tauri (Svelte ↔ Rust via IPC). Chat traffic uses raw TCP sockets only. The server routes private and group messages, runs an AI chatbot in group rooms, and relays typing indicators and read receipts. Clients may run on different machines; the server can run on one of them or a third host.

```
┌─────────────────────────────────┐         TCP          ┌─────────────────┐
│  Tauri client                   │ ───────────────────► │  Rust server    │
│  • Svelte UI                    │◄───────────────────  │  • framing      │
│  • Rust TCP + #[tauri::command] │                      │  • routing      │
└─────────────────────────────────┘                      │  • groups / AI  │
                                                         └─────────────────┘
```

## Features

| ID | Course | Feature | Description |
|----|--------|---------|-------------|
| F1 | R3 | Unique names | Each client registers a unique display name on connect. |
| F2 | R4 | Active users | Every client sees the list of currently connected names (including themselves). |
| F3 | R5 | Chat rooms | Private and group conversations each get their own room. |
| F4 | R6 | Chat UI | Each room has a message window and a box to send text. |
| F5 | R7 | Private chat | Direct messages between two clients; only those two can see them. |
| F6 | R8 | Create group | A client can create a group; it starts with only themselves as a member. |
| F7 | R9 | Group list | Clients can see all groups and each group’s members. |
| F8 | R10 | Join group | Clients join groups themselves; creators do not auto-add others. |
| F9 | R11 | Group chat | Group messages are visible only to members of that group. |
| F10 | Part 3 | AI group chatbot | An AI chatbot (e.g. OpenAI / Ollama) participates in group chat. |
| F11 | Part 3 | Read / unread | Each room shows an unread count for messages you have not opened yet, and senders see when their messages have been read. |
| F12 | Part 3 | Typing indicator | A room shows who is currently typing (private and group). |
| F13 | Part 3 | Leave / delete group | A member can leave a group; the group's creator can delete it for everyone. |

## Getting started

### Prerequisites

- Rust (stable) + Cargo
- Node.js + pnpm
- System deps for Tauri (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

### Server

```bash
cd server
cargo run
```

### Client (Tauri GUI)
spawn client in a window

```bash
cd client
pnpm install
pnpm tauri dev
```

# Testing for dev
Spawn 2 client + 1 server
```bash
chmod +x dev.sh
./dev.sh
```

## AI model
Option 1: local ollama
```bash
# Install
curl -fsSL https://ollama.com/install.sh | sh   # install Ollama
ollama pull llama3.2:1b                         # (~1.3 GB)
# Test
cargo test -p server ai -- --ignored --nocapture   # asks Ollama once, prints a pirate reply
# Run
ollama serve # wiring ollama to stand by in memory
```

Option 2: use external sevice API \

Register
- Get an account ID: sign up at dash.cloudflare.com. The account ID is shown on the account home page and on the Workers AI page.
- Get an API token: go to My Profile → API Tokens → Create Token and pick the "Workers AI" template. Copy the token; it's only shown once.
- Put this in .env, with your account ID in the URL
```txt
DEEPCHAT_AI_URL=https://api.cloudflare.com/client/v4/accounts/<ACCOUNT_ID>/ai/v1
DEEPCHAT_AI_MODEL=@cf/meta/llama-3.1-8b-instruct
DEEPCHAT_AI_KEY=<API_TOKEN>
```


## Repo layout

```
Deepchat/
  server/     # TCP chat server (Rust)
  client/     # Tauri desktop client (Svelte UI + Rust IPC / TCP)
```
