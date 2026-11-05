---
name: Deepchat
description: A minimal Slack-style LAN chat client set in deep-ocean navy with one flat aqua accent.
colors:
  bg: "#0e1a33"
  side: "#0a1428"
  raise: "#15264a"
  raise-hover: "#1b2e57"
  line: "#1f3359"
  line-strong: "#2b4372"
  ink: "#e8eef8"
  text: "#d5deec"
  mute: "#a3b3cd"
  faint: "#7f92b1"
  accent: "#4fd8eb"
  accent-hover: "#80e5f2"
  accent-ink: "#071224"
  accent-soft: "rgb(79 216 235 / 0.14)"
  ok: "#3ddc97"
  err: "#ff7b7b"
  err-soft: "rgb(255 123 123 / 0.12)"
  avatar-1: "#34548c"
  avatar-2: "#2f6690"
  avatar-3: "#56657f"
  avatar-4: "#4b5363"
  avatar-5: "#94681a"
  avatar-6: "#8a5a2b"
  avatar-7: "#6f6a2e"
  avatar-8: "#7a6a52"
typography:
  headline:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "24px"
    fontWeight: 650
    lineHeight: 1.2
    letterSpacing: "-0.02em"
    fontFeature: "\"cv11\", \"ss01\""
  title:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 650
    letterSpacing: "-0.01em"
    fontFeature: "\"cv11\", \"ss01\""
  body:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    fontFeature: "\"cv11\", \"ss01\""
  label:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    fontFeature: "\"cv11\", \"ss01\""
  caption:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    fontFeature: "\"cv11\", \"ss01\""
  badge:
    fontFamily: "Inter Variable, system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 700
    fontFeature: "\"cv11\", \"ss01\""
rounded:
  sm: "6px"
  md: "8px"
  lg: "10px"
  pill: "10px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
  xxl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    rounded: "{rounded.md}"
    height: "40px"
    padding: "0 20px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
  button-primary-disabled:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.faint}"
  button-primary-compact:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    rounded: "{rounded.sm}"
    height: "32px"
    padding: "0 12px"
  button-ghost:
    textColor: "{colors.mute}"
    rounded: "{rounded.sm}"
    height: "32px"
    padding: "0 12px"
  button-ghost-hover:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.ink}"
  button-join-inline:
    backgroundColor: "{colors.side}"
    textColor: "{colors.accent}"
    rounded: "{rounded.sm}"
    height: "24px"
    padding: "0 10px"
  button-send:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    size: "32px"
  button-send-disabled:
    backgroundColor: "{colors.raise-hover}"
    textColor: "{colors.faint}"
  input-field:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    height: "40px"
    padding: "0 12px"
  composer:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    padding: "6px 6px 6px 14px"
  sidebar-item:
    textColor: "{colors.mute}"
    rounded: "{rounded.sm}"
    height: "32px"
    padding: "0 8px"
  sidebar-item-hover:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.ink}"
  sidebar-item-active:
    backgroundColor: "{colors.accent-soft}"
    textColor: "{colors.accent}"
  unread-pill:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    typography: "{typography.badge}"
    rounded: "{rounded.pill}"
    height: "20px"
    padding: "0 6px"
  avatar:
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    size: "32px"
  avatar-small:
    textColor: "#ffffff"
    rounded: "{rounded.sm}"
    size: "24px"
  toast:
    backgroundColor: "{colors.raise}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    padding: "12px 8px 12px 12px"
    width: "360px"
  popover:
    backgroundColor: "{colors.raise}"
    rounded: "{rounded.lg}"
    padding: "6px"
    width: "240px"
  banner-error:
    backgroundColor: "{colors.err-soft}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "10px 12px"
---

# Design System: Deepchat

## Overview

**Creative North Star: "Deep Water, One Light"**

Deepchat is a category-standard chat client played straight: one quiet sidebar, one message column, nothing decorative. The world is a rich deep-ocean navy that never falls to black, layered in three tonal steps (sidebar deeper than the room, raised fields lighter than both) and divided by 1px hairlines. A single flat aqua is the only light in the water; it marks actions, the current selection, and unread counts, always carrying navy text on top and never glowing.

Density is desktop-chat density: 32px rows, 56px headers, 14px body text, a 264px sidebar. Everything protocol-backed is visible at a glance; nothing is hidden behind novelty. The direction rejected novel metaphors (a transit-map concept) and the purple/glow AI-app look.

Errors are placed where the action happened, never in a generic global slot unless no local place exists.

**Key Characteristics:**
- Navy tonal layering, no black, no gradients.
- One flat aqua accent with navy ink on it; no glow.
- Green means online and nothing else; red means error and nothing else.
- Soft 6–10px corners, 1px lines, bundled Inter Variable.
- Left-aligned message feed with hashed-colour initial avatars.

## Colors

A cold, dark navy family with one luminous aqua and two strictly semantic signals.

### Primary
- **Lagoon Aqua** (`accent`): primary buttons, the send button, unread pills, the active sidebar row's text, inline Join text, focus outlines and field focus borders. `accent-hover` lightens it on hover; `accent-ink` is the navy text that sits on it; `accent-soft` is the translucent wash behind the active sidebar row, focus rings and inline Join hover.

### Neutral
- **Abyss Navy** (`bg`): the room, connect screen and app canvas.
- **Trench Navy** (`side`): the sidebar, one step deeper than the room; also the inline Join button's fill.
- **Shoal Navy** (`raise`): inputs, the composer, popovers, toasts, and row hover.
- **Shoal Navy Lit** (`raise-hover`): hover inside an already-raised surface (toast close button) and the disabled send button.
- **Hairline** (`line`): structural dividers: header bottoms, sidebar edge, day separators, preview member rules.
- **Strong Hairline** (`line-strong`): borders of interactive or floating surfaces: inputs, composer, popovers, toasts, inline Join; scrollbar thumb.
- **Foam Ink** (`ink`): primary text, names, headings, hovered/unread row text.
- **Message Text** (`text`): message body copy only, a half-step softer than ink for long reading.
- **Mist** (`mute`): secondary text: header meta, hint lines, resting sidebar rows, ghost buttons.
- **Deep Mist** (`faint`): tertiary text: section headings, timestamps, "you" tags, placeholders, `#` glyphs, non-member groups, disabled text.
- **Avatar fills** (`avatar-1`…`avatar-8`): eight blues, slates and earth tones chosen per name by an FNV-1a hash; white initials stay above 4.5:1 on each. No red, aqua, green or purple.

### Semantic
- **Online Green** (`ok`): presence dots only (6px in the identity footer, 8px in lists and DM header).
- **Signal Red** (`err`): error text, invalid-field borders, alert icons. `err-soft` backs the disconnect banner.

### Named Rules
**The One Light Rule.** Aqua is the only accent. It fills actions and unread counts, tints the selection, and is never used as a glow or gradient.

**The Semantic Lock Rule.** Green is presence, red is error, aqua is selection/action/unread. None appears decoratively, and avatar fills stay within blues, slates and earth tones.

**The Never Black Rule.** The darkest surface is Trench Navy; depth comes from navy steps, not black.

## Typography

**Display Font:** none (no display face)
**Body Font:** Inter Variable (with system-ui, sans-serif), bundled via @fontsource-variable/inter, features `cv11` and `ss01` on.

**Character:** One neutral grotesque at desktop-chat sizes; hierarchy comes from weight (400 / 500 / 600 / 650 / 700) and the ink → mute → faint colour ladder rather than size jumps.

### Hierarchy
- **Headline** (650, 24px, 1.2, -0.02em): the connect screen's single heading.
- **Title** (650, 16px, -0.01em on the wordmark): the "Deepchat" wordmark and room / preview header names.
- **Body** (400, 14px, 1.5): everything by default; message bodies cap at 72ch.
- **Label** (600, 13px): form labels; 13px also carries header meta, ghost buttons (500), banners and toasts (line-height 1.45).
- **Caption** (12px): section headings (600, faint, sentence case), timestamps, "you" tags, inline errors (line-height 1.4), the signed-in line.
- **Badge** (700, 11px, tabular numbers): unread pills; 11px also used for small-avatar initials and member-panel tags.

### Named Rules
**The Weight Not Size Rule.** Distinguish by weight and the ink/mute/faint ladder; keep sizes within 11–16px inside the app, with 24px reserved for the connect heading.

**The Tabular Count Rule.** Counts, timestamps and host:port use `font-variant-numeric: tabular-nums`.

## Layout

A fixed two-column shell: a 264px sidebar and a fluid main column, full viewport height. The sidebar is a three-row grid: 56px brand row, scrolling lists, and an identity footer (avatar, "Signed in as", name, green dot + host:port, Disconnect icon button). Group and user sections are padded 16px 8px with 28px headings and 32px rows separated by 1px gaps.

The room is a three-row grid: a 56px header (24px left padding, title, meta, actions pushed right), a scrolling feed bottom-anchored with `justify-content: flex-end`, and the composer pinned at the bottom (padding 4px 24px 20px). Messages sit 24px from the edges with a 12px gap between a 32px avatar and the body; consecutive messages from the same sender collapse into a 32px gutter that reveals the time on hover. Day separators are centred faint labels between hairlines.

The connect screen centres a 360px form with a fixed top offset of `max(32px, 22vh)` so banners and errors grow downward without moving the heading; the wordmark sits absolutely at 16px/16px. The group preview uses 32px 24px padding with content capped at 420px.

Spacing rhythm runs on 4px: 4, 6, 8, 10, 12, 16, 20, 24, 28, 32, 40. No breakpoints are defined; the shell is a desktop/Tauri window layout. Minimum window size is still open.

## Elevation & Depth

Flat by default; depth is tonal (side → bg → raise) plus 1px hairlines. A single shadow exists, and only on surfaces that float above the layout: the Members popover and toasts (`box-shadow: 0 12px 32px rgb(3 8 20 / 0.5)`). Focused fields add a 3px `accent-soft` ring, which is a state, not elevation.

### Named Rules
**The Float-Only Shadow Rule.** Only overlays that leave the layout flow (popover, toast) cast the shadow; rows, cards and fields never do.

## Shapes

Soft rectangles throughout. Small controls (sidebar rows, ghost and icon buttons, compact inputs, inline Join, 24px avatars) use `sm`; standard fields, primary buttons, 32px avatars and banners use `md`; containers that hold other things (composer, popover, toast) use `lg`. Unread pills are fully rounded at a 20px height. Presence dots are circles; the offline DM dot is a 1.5px faint inset ring. All borders are 1px. Icons are 16×16 line glyphs on a 16-unit viewBox, 1.5 stroke, round caps and joins, in currentColor.

## Components

### Buttons
- **Shape:** gently rounded (`md` for 40px buttons and the send button, `sm` for other 32px-and-smaller buttons).
- **Primary:** aqua fill, navy ink, weight 650; 40px tall on the connect form and group preview Join, 32px compact in the create-group box.
- **Hover / Focus:** fill lightens to accent-hover with a 120ms ease background transition; keyboard focus shows the global 2px aqua outline offset 2px.
- **Disabled:** raise fill with faint text, `not-allowed` cursor.
- **Ghost:** transparent, mute text, 13px/500, 32px tall with icon + label (Members, Leave); hover and `aria-expanded` fill with raise and ink.
- **Icon button:** 28–32px square, transparent, mute; hover raise + ink (Create group +, Disconnect).
- **Inline Join:** 24px outlined chip on non-member group rows: side fill, strong-hairline border, aqua 12px/600 text; hover turns border aqua and fill accent-soft.
- **Send:** 32px aqua square (`md` radius) inside the composer with an up-arrow icon; disabled uses raise-hover and faint, because a raise fill would vanish against the raise-filled composer.

### Inputs / Fields
- **Style:** raise fill, 1px strong hairline, ink text, faint placeholder; 40px on connect, 32px compact in create-group.
- **Focus:** border turns aqua plus a 3px accent-soft ring, 120ms ease.
- **Error / Disabled:** `aria-invalid` turns the border red with a 12px red message directly under the field; disabled fields drop to 0.6 opacity.

### Composer
A 10px-radius raised bar holding a borderless auto-growing textarea (one line at rest, up to 144px, then it scrolls) and the send button, bottom-aligned; `focus-within` gives the whole bar the aqua border and ring. Enter sends and Shift+Enter inserts a newline; Enter is ignored while an IME is composing. Send errors render as a red 12px line with an alert icon above the bar, and the draft is kept. When a DM peer is offline a mute note "{name} is offline." takes the same slot. Disabled state drops the bar to 0.6 opacity.

### Navigation (Sidebar Lists)
- **Rows:** 32px, `sm` radius, transparent. Groups lead with a `#` glyph; users lead with an 8px green dot.
- **States:** member groups and users rest at mute, non-member groups at faint; hover raise + ink; unread rows go ink and 600; the active row gets an accent-soft wash, aqua text, 600.
- **Unread pill:** 20px-tall aqua pill with navy 11px/700 tabular count, right-aligned.
- **Section heads:** 12px/600 faint, sentence case ("Groups", "Active users") with a + icon or tabular total on the right.
- **Self:** your own row carries a faint "you" tag and a default cursor.

### Message
Left-aligned for everyone, including you. 32px initial avatar (hashed fill, white 13px/700 initial), a header of name (600) + faint "you" tag + faint tabular time, then body text in the `text` colour, pre-wrapped, capped at 72ch. Row hover adds a near-invisible white wash. Continuations drop the avatar and header.

### Membership Notice
A faint 12px line in the message feed, "mya joined the group" or "mya left the group" with a faint time, name at 600. It sits on the message text column (68px left, aligned with message bodies) with no avatar, tinted `faint` so it never competes with messages. Derived client-side from group snapshots, shown only for groups you belong to, kept in memory (lost on reconnect), and never counted as unread. A notice breaks message grouping, so the next message shows its header again.

### Typing Indicator
One mute 12px line directly above the composer: "mya is typing…", "mya and bo are typing…", or "Several people are typing…" for three or more. It shares the send-note slot, so a send error or the "is offline" note replaces it. Shown only in the room the signal belongs to, never for yourself. Each signal keeps a name visible for 4s; a message from that person ends it at once. Exposed as a polite status region.

### Room Header
56px bar with `#` glyph (faint) + 16px/650 title, 13px mute meta ("3 members · Created by mya", or online/offline status with a dot for DMs), and ghost actions on the right. Members opens a 240px popover (raise, strong hairline, `lg`, float shadow) listing 36px rows with 24px avatars and faint 11px tags.

### Group Preview (Signature)
Non-member groups open a preview instead of an error: header with title and meta, a mute lead line, a 40px primary Join, then a faint "Members" heading over a hairline-ruled list of 44px rows with 24px avatars and faint 12px tags.

### Toasts
Top-right stack (68px from top, 16px from right, below the header), up to 3, width `min(360px, 100vw - 32px)`, 8px gap. Each toast: raise fill, strong hairline, `lg`, float shadow, red alert icon, 13px ink text, 24px ghost close button. Slides in 24px from the right over 240ms cubic-out and out in 60% of that; reduced motion removes both. Auto-dismiss after 6s, paused on hover, identical messages restart the existing timer instead of stacking.

### Connect Banner
After a disconnect the connect screen shows an err-soft banner (`md`, 10px 12px, 13px ink text, red alert icon) above prefilled fields.

### Error Placement
- Connect errors: inline under the offending field.
- Create-group errors: inside the create box, spanning both columns.
- Send errors: above the composer, draft kept.
- Other async server errors: toasts.
- Disconnect: back to the connect screen with the banner and fields prefilled.

### Specified, Not Yet Implemented
These slots are part of the system but **are not built**; each needs protocol support first. Do not ship them as stubs.
- **Read receipts:** "Seen by …" in faint 12px directly under your most recent message.
- **AI bot member:** avatar filled with aqua (navy initial) plus a small outlined "AI" tag after the name, wherever members appear.

## Do's and Don'ts

### Do:
- **Do** layer surfaces side → bg → raise and separate them with 1px hairlines.
- **Do** put navy `accent-ink` text on every aqua fill.
- **Do** place errors where the action happened: under the field, inside the create box, above the composer; toasts only for errors with no local home.
- **Do** keep counts, times and host:port tabular.
- **Do** honour reduced motion by zeroing movement and duration.

### Don't:
- **Don't** use black, purple, gradients or glow; the direction rejected the purple/glow AI-app look.
- **Don't** use green for anything but presence or red for anything but errors.
- **Don't** cast shadows on in-flow surfaces; only the popover and toasts float.
- **Don't** add a second accent colour.
- **Don't** right-align your own messages into bubbles; the feed is left-aligned for everyone.
