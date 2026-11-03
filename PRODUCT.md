# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Tauri 2 desktop application. `web` is the closest schema value (HTML/CSS/Svelte in a webview); this is not a browser app.

## Users
Course demo audience: an instructor and classmates evaluating a computer-networks socket-programming project. They run the client on a few machines on one LAN, connect to a shared server, and exercise the feature list (registration, rooms, private/group chat, AI bot, read receipts, typing indicators) in a short demo.

## Product Purpose
Deepchat is a multi-client chat application built on raw TCP sockets in Rust, with a Tauri desktop client (Svelte + TypeScript frontend, Rust backend over IPC). The server routes private and group messages, relays typing indicators and read receipts, and runs an AI chatbot in group rooms. Success: every feature F1–F13 in README.md is visible and demonstrable in the UI during a live demo.

## Positioning
An AI chatbot that participates inside group rooms as a conversation member, on a self-hosted raw-TCP chat stack (no cloud). Other chat apps do not have a bot that is a member of a user-created group on a server the user runs themselves.

## Operating Context
- Clients on different machines on one LAN; server runs on one of them or a third host.
- Connect flow: pick a unique display name (F1), connect to the server, see the active user list (F2).
- Rooms: private DMs and groups, each with message window and send box (F3–F9); create, list, join, leave, delete groups (F6–F8, F13).
- Presence: per-room unread counts and read receipts (F11), typing indicators (F12).
- Desktop window currently 1280×720 (client/src-tauri/tauri.conf.json).
- Frontend entry: client/src/routes/+page.svelte; components ConnectView, UserList, GroupList, ChatRoom in client/src/lib/components; IPC in client/src/lib/chatApi.ts.

## Capabilities and Constraints
- Must preserve all logic and IPC contracts (chatApi.ts, types.ts, Rust backend); redesign is presentation only.
- Transport is raw TCP, owned by the Rust side. The frontend never opens a socket (no WebSocket/HTTP to the server); it only uses Tauri `invoke` commands and events. The UI runs only inside the Tauri window; a plain browser has no backend.
- Must work offline / LAN only: no remote fonts, CDNs, or network-loaded assets.
- Stack is fixed: SvelteKit (static adapter) + Svelte 5 + TypeScript, Tauri 2, pnpm.
- Terminology: "Deepchat", rooms, groups, private chat, active users, unread, typing, read receipts.
- Undecided: light/dark theme support; minimum window size.

## Brand Commitments
App name "Deepchat" and existing UI wording stay. Visual identity is explicitly open and is to be replaced from scratch (user-requested redesign; the old look is not binding).

Binding preference (user, 2026-11-03): minimal, easy to use, UX first. Rejected an expressive "transit map" direction as too complicated. Familiar chat conventions (Slack/Discord-style layout) are welcome; decoration and novel metaphors are not. Avoid the purple/glow AI-app look.

## Evidence on Hand
README.md feature table F1–F13; docs/protocol.md; docs/server-architecture.md. No logo, screenshots, testimonials or brand assets found. Do not invent any.

## Product Principles
- Every feature in F1–F13 must be discoverable at a glance in a live demo, with nothing hidden behind secondary menus.
- The AI bot is a visible participant in group rooms, not a hidden feature.
- Live state (who is online, who is typing, what is unread or read) is the product's core signal and must read instantly.
- Works entirely offline on a LAN; no external dependencies at runtime.
- Presentation changes never alter protocol or backend behavior.
