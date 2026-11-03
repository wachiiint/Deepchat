<script lang="ts">
  import type { ActiveRoom, Group, Message } from "$lib/types";
  import { avatarFill, initial } from "$lib/avatar";
  import Icon from "./Icon.svelte";

  let {
    room = null,
    group = null,
    messages = [],
    currentUser = "",
    busy = false,
    peerOnline = true,
    sendError = "",
    onsend,
    onleave,
    oneditdraft,
  }: {
    room?: ActiveRoom | null;
    group?: Group | null;
    messages?: Message[];
    currentUser?: string;
    busy?: boolean;
    peerOnline?: boolean;
    sendError?: string;
    onsend: (text: string) => Promise<boolean>;
    onleave: (groupId: string) => void | Promise<void>;
    oneditdraft: () => void;
  } = $props();

  let draft = $state("");
  let windowEl: HTMLDivElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();
  let membersOpen = $state(false);
  let membersEl: HTMLDivElement | undefined = $state();

  const GROUP_GAP_MS = 5 * 60 * 1000;
  const canSend = $derived(!!room && peerOnline);

  type Row = { message: Message; continued: boolean; day: string | null };

  const rows = $derived.by(() => {
    const out: Row[] = [];
    let prev: Message | null = null;
    let prevDay = "";
    for (const message of messages) {
      const day = new Date(message.timestamp).toDateString();
      const newDay = day !== prevDay;
      const continued =
        !newDay &&
        prev !== null &&
        prev.sender === message.sender &&
        message.timestamp - prev.timestamp < GROUP_GAP_MS;
      out.push({
        message,
        continued,
        day: newDay ? dayLabel(message.timestamp) : null,
      });
      prev = message;
      prevDay = day;
    }
    return out;
  });

  $effect(() => {
    messages;
    if (windowEl) windowEl.scrollTop = windowEl.scrollHeight;
  });

  $effect(() => {
    room?.id;
    membersOpen = false;
    draft = "";
    if (room) inputEl?.focus();
  });

  function dayLabel(ts: number): string {
    const d = new Date(ts);
    const today = new Date();
    const yesterday = new Date();
    yesterday.setDate(today.getDate() - 1);
    if (d.toDateString() === today.toDateString()) return "Today";
    if (d.toDateString() === yesterday.toDateString()) return "Yesterday";
    return d.toLocaleDateString([], {
      weekday: "long",
      month: "short",
      day: "numeric",
    });
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  async function handleSend(event: Event) {
    event.preventDefault();
    const text = draft.trim();
    if (!text || !canSend) return;
    if (await onsend(text)) draft = "";
    inputEl?.focus();
  }

  function handleWindowClick(event: MouseEvent) {
    if (membersOpen && membersEl && !membersEl.contains(event.target as Node)) {
      membersOpen = false;
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && membersOpen) membersOpen = false;
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<section class="room">
  {#if !room}
    <p class="empty-state">Select a user or group to open a chat room.</p>
  {:else}
    <header class="head">
      <h1>
        <span class="glyph"
          ><Icon name={room.kind === "group" ? "hash" : "user"} /></span
        >
        <span class="title">{room.title}</span>
      </h1>
      {#if room.kind === "group" && group}
        <p class="meta">
          {group.members.length} member{group.members.length === 1 ? "" : "s"} · Created
          by {group.creator}
        </p>
      {:else}
        <p class="meta status" class:offline={!peerOnline}>
          <span class="dot" aria-hidden="true"></span>{peerOnline
            ? "Private · online"
            : "Private · offline"}
        </p>
      {/if}

      {#if room.kind === "group" && group}
        <div class="actions">
          <div class="members" bind:this={membersEl}>
            <button
              type="button"
              class="ghost"
              aria-expanded={membersOpen}
              aria-controls="members-panel"
              onclick={() => (membersOpen = !membersOpen)}
            >
              <Icon name="users" />Members
            </button>
            {#if membersOpen}
              <div
                class="panel"
                id="members-panel"
                role="dialog"
                aria-label="Members"
              >
                <ul>
                  {#each group.members as member (member)}
                    <li>
                      <span class="av sm" style:background={avatarFill(member)}
                        >{initial(member)}</span
                      >
                      <span class="name">{member}</span>
                      {#if member === currentUser}<span class="tag">you</span
                        >{/if}
                      {#if member === group.creator}<span class="tag"
                          >creator</span
                        >{/if}
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
          <button type="button" class="ghost" onclick={() => onleave(group.id)}
            >Leave</button
          >
        </div>
      {/if}
    </header>

    <div class="window" bind:this={windowEl}>
      <div class="feed">
        {#each rows as { message, continued, day } (message.id)}
          {#if day}
            <p class="day"><span>{day}</span></p>
          {/if}
          <article class="msg" class:continued>
            {#if continued}
              <span class="gutter"
                ><time>{formatTime(message.timestamp)}</time></span
              >
            {:else}
              <span
                class="av"
                style:background={avatarFill(message.sender)}
                aria-hidden="true"
              >
                {initial(message.sender)}
              </span>
            {/if}
            <div class="body">
              {#if !continued}
                <header>
                  <strong>{message.sender}</strong>
                  {#if message.sender === currentUser}<span class="you"
                      >you</span
                    >{/if}
                  <time>{formatTime(message.timestamp)}</time>
                </header>
              {/if}
              <p>{message.text}</p>
            </div>
          </article>
        {:else}
          <p class="empty-msg">No messages yet. Say hello.</p>
        {/each}
      </div>
    </div>

    <form class="composer" onsubmit={handleSend}>
      {#if sendError}
        <p class="send-error" role="alert">
          <Icon name="alert" size={14} />{sendError}
        </p>
      {:else if !peerOnline}
        <p class="send-note">{room.title} is offline.</p>
      {/if}
      <div class="box" class:disabled={!canSend}>
        <input
          bind:this={inputEl}
          bind:value={draft}
          aria-label="Message"
          placeholder="Type a message…"
          disabled={!canSend}
          autocomplete="off"
          oninput={() => sendError && oneditdraft()}
        />
        <button
          type="submit"
          class="send"
          aria-label="Send"
          title="Send"
          disabled={busy || !canSend || !draft.trim()}
        >
          <Icon name="send" />
        </button>
      </div>
    </form>
  {/if}
</section>

<style>
  .room {
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr auto;
    min-height: 0;
  }

  .empty-state {
    grid-row: 1 / -1;
    place-self: center;
    color: var(--mute);
  }

  .head {
    height: 56px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px 0 24px;
    border-bottom: 1px solid var(--line);
    min-width: 0;
  }

  h1 {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 16px;
    font-weight: 650;
    min-width: 0;
  }

  .glyph {
    color: var(--faint);
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    color: var(--mute);
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
  }

  .status.offline .dot {
    background: none;
    box-shadow: inset 0 0 0 1.5px var(--faint);
  }

  .actions {
    margin-left: auto;
    display: flex;
    gap: 4px;
    flex: none;
  }

  .ghost {
    height: 32px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
    font-size: 13px;
    font-weight: 500;
  }

  .ghost:hover,
  .ghost[aria-expanded="true"] {
    background: var(--raise);
    color: var(--ink);
  }

  .members {
    position: relative;
  }

  .panel {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 10;
    width: 240px;
    max-height: 320px;
    overflow-y: auto;
    padding: 6px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--raise);
    box-shadow: 0 12px 32px rgb(3 8 20 / 0.5);
  }

  .panel ul {
    list-style: none;
    padding: 0;
  }

  .panel li {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 36px;
    padding: 0 8px;
  }

  .panel .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    font-size: 11px;
    color: var(--faint);
  }

  .window {
    overflow-y: auto;
    min-height: 0;
  }

  .feed {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding: 16px 0 8px;
  }

  .day {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    color: var(--faint);
    font-size: 12px;
    font-weight: 600;
  }

  .day::before,
  .day::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--line);
  }

  .msg {
    display: flex;
    gap: 12px;
    padding: 6px 24px 2px;
  }

  .msg.continued {
    padding-top: 0;
  }

  .msg:hover {
    background: rgb(255 255 255 / 0.025);
  }

  .av {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    display: grid;
    place-items: center;
    flex: none;
    color: #fff;
    font-weight: 700;
    font-size: 13px;
    margin-top: 2px;
  }

  .av.sm {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    margin-top: 0;
  }

  .gutter {
    width: 32px;
    flex: none;
    display: flex;
    justify-content: flex-end;
    padding-top: 3px;
  }

  .gutter time {
    visibility: hidden;
    font-size: 10px;
  }

  .msg:hover .gutter time {
    visibility: visible;
  }

  .body {
    min-width: 0;
  }

  .body header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .body strong {
    font-weight: 600;
  }

  .you {
    font-size: 12px;
    color: var(--faint);
  }

  time {
    font-size: 12px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }

  .body p {
    max-width: 72ch;
    color: var(--text);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .empty-msg {
    padding: 24px;
    color: var(--faint);
    text-align: center;
  }

  .composer {
    padding: 4px 24px 20px;
  }

  .send-error,
  .send-note {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 2px 8px;
    font-size: 12px;
  }

  .send-error {
    color: var(--err);
  }

  .send-note {
    color: var(--mute);
  }

  .box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 6px 6px 14px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--raise);
    transition:
      border-color 120ms ease,
      box-shadow 120ms ease;
  }

  .box:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .box.disabled {
    opacity: 0.6;
  }

  .box input {
    flex: 1;
    min-width: 0;
    height: 32px;
    border: 0;
    outline: none;
    background: none;
    color: var(--ink);
  }

  .box input::placeholder {
    color: var(--faint);
  }

  .send {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-ink);
    transition:
      background-color 120ms ease,
      opacity 120ms ease;
  }

  .send:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .send:disabled {
    background: var(--raise-hover);
    color: var(--faint);
  }
</style>
