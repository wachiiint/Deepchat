# Deepchat

A multi-client chat application built with **TCP socket programming in Rust** and a **Tauri** desktop GUI (Svelte + TypeScript frontend, Rust client backend).

## Overview

Clients talk to a Rust TCP server; the desktop UI is Tauri (Svelte ↔ Rust via IPC). Chat traffic uses raw TCP sockets only. The server routes private and group messages, runs an AI chatbot and on-demand AI summaries in group rooms, and relays typing indicators. Clients may run on different machines; the server can run on one of them or a third host.

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
| F11 | Part 3 | AI chat summary | A group member can ask for an AI-generated summary of the group's recent messages. |
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

```bash
cd client
pnpm install
pnpm tauri dev
```

## Repo layout

```
Deepchat/
  server/     # TCP chat server (Rust)
  client/     # Tauri desktop client (Svelte UI + Rust IPC / TCP)
```
