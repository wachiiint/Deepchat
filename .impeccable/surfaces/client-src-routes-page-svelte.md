---
version: 1
slug: "client-src-routes-page-svelte"
primary_target: "client/src/routes/+page.svelte"
related_targets: []
---

## Scope

Whole Deepchat client UI (connect screen, app shell, sidebar lists, chat room, errors). Mode: Operate. Code-led.

## Audience and task

Course demo audience on a LAN and a classroom projector. Connect with a unique name, see who is online, open private chats, create/join/leave groups, chat. Every protocol-backed feature (F1–F9, leave) visible at a glance.

## Direction contract

THESIS: A minimal Slack-style chat client in deep water: one quiet sidebar, one message column, nothing decorative. Refuses novel metaphors (rejected the transit-map direction) and the purple/glow AI-app look.

OWN-WORLD: Rich deep-ocean navy, never black: main #0e1a33, sidebar one step deeper #0a1428, raised fields #15264a, hairlines #1f3359. Near-white ink, blue-grey secondary text. One flat aqua accent #4fd8eb for actions, selection and unread, with navy text on it; no glow. Green for online, red only for errors. Rounded 6–10px corners, 1px lines, bundled Inter.

STORY: The visitor sees their name and server, the groups and active users, opens a room, reads and sends. Errors appear where the action happened.

FIRST VIEWPORT: 264px sidebar (Deepchat, Groups with +, Active users with green dots, you + Disconnect at the bottom). Main: room header (name, members, creator, Members, Leave), messages left-aligned with initial avatars, composer pinned bottom with aqua send.

FORM: seed:c21f854b708a813a, re-rolled safer by the user to the category standard, played straight with a deep-navy palette.

SIGNATURE: Unread counts as aqua pills; non-member groups open a preview (creator, members, Join) instead of erroring.

## Error placement (confirmed)

Connect errors inline under the field; create-group errors inside the create box; send errors above the composer with draft kept; other async server errors as top-right toasts below the header (auto-dismiss ~6s, pause on hover, max 3, deduped); disconnects return to the connect screen with a banner and fields prefilled.

## Open

Minimum window size; light theme (not planned); typing, read receipts, AI bot need protocol support first.
