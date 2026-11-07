<script lang="ts">
  import type { ActiveRoom, Group, Message, RoomEvent } from "$lib/types";
  import { avatarFill, initial } from "$lib/avatar";
  import { createBot, removeBot } from "$lib/chatApi";
  import Icon from "./Icon.svelte";

  let {
    room = null,
    group = null,
    messages = [],
    events = [],
    typers = [],
    currentUser = "",
    busy = false,
    peerOnline = true,
    sendError = "",
    onsend,
    ontyping,
    onleave,
    ondelete,
    oneditdraft,
  }: {
    room?: ActiveRoom | null;
    group?: Group | null;
    messages?: Message[];
    events?: RoomEvent[];
    typers?: string[];
    currentUser?: string;
    busy?: boolean;
    peerOnline?: boolean;
    sendError?: string;
    onsend: (text: string) => Promise<boolean>;
    ontyping: () => void;
    onleave: (groupId: string) => void | Promise<void>;
    ondelete: (group: Group) => void;
    oneditdraft: () => void;
  } = $props();

  let draft = $state("");
  let windowEl: HTMLDivElement | undefined = $state();
  let inputEl: HTMLTextAreaElement | undefined = $state();
  let membersOpen = $state(false);
  let membersEl: HTMLDivElement | undefined = $state();
  let botName = $state("");
  let botPersonality = $state("");

  // Server errors (limit, name taken) come back as chat-error toasts.
  async function addBot(event: Event) {
    event.preventDefault();
    if (!group || !botName.trim() || !botPersonality.trim()) return;
    await createBot(group.id, botName, botPersonality);
    botName = "";
    botPersonality = "";
  }

  // At most one typing signal per throttle window; the receiver's lease is longer.
  const TYPING_THROTTLE_MS = 2000;
  let lastTypingAt = 0;

  const GROUP_GAP_MS = 5 * 60 * 1000;
  const canSend = $derived(!!room && peerOnline);
  const typingLabel = $derived(
    typers.length === 0
      ? ""
      : typers.length === 1
        ? `${typers[0]} is typing…`
        : typers.length === 2
          ? `${typers[0]} and ${typers[1]} are typing…`
          : "Several people are typing…",
  );

  type Row =
    | {
        kind: "message";
        key: string;
        message: Message;
        continued: boolean;
        day: string | null;
      }
    | { kind: "event"; key: string; event: RoomEvent; day: string | null };

  const rows = $derived.by(() => {
    // Messages first on a tie so "mya joined" never precedes the message that caused it.
    const timeline = [
      ...messages.map((message) => ({ at: message.timestamp, message })),
      ...events.map((event) => ({ at: event.timestamp, event })),
    ].sort((a, b) => a.at - b.at);

    const out: Row[] = [];
    let prev: Message | null = null;
    let prevDay = "";
    for (const item of timeline) {
      const day = new Date(item.at).toDateString();
      const dayLabelText = day !== prevDay ? dayLabel(item.at) : null;
      prevDay = day;
      if ("event" in item) {
        out.push({
          kind: "event",
          key: `event:${item.event.id}`,
          event: item.event,
          day: dayLabelText,
        });
        prev = null;
        continue;
      }
      const { message } = item;
      const continued =
        dayLabelText === null &&
        prev !== null &&
        prev.sender === message.sender &&
        message.timestamp - prev.timestamp < GROUP_GAP_MS;
      out.push({
        kind: "message",
        key: message.id,
        message,
        continued,
        day: dayLabelText,
      });
      prev = message;
    }
    return out;
  });

  $effect(() => {
    rows;
    if (windowEl) windowEl.scrollTop = windowEl.scrollHeight;
  });

  $effect(() => {
    room?.id;
    membersOpen = false;
    draft = "";
    lastTypingAt = 0;
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

  // Grow with the draft up to the CSS max-height; shrink back when cleared.
  $effect(() => {
    draft;
    if (!inputEl) return;
    inputEl.style.height = "auto";
    inputEl.style.height = `${inputEl.scrollHeight}px`;
  });

  async function submit() {
    const text = draft.trim();
    if (!text || !canSend) return;
    if (await onsend(text)) {
      draft = "";
      lastTypingAt = 0; // the next keystroke after a send signals at once
    }
    inputEl?.focus();
  }

  function handleSubmit(event: Event) {
    event.preventDefault();
    void submit();
  }

  function handleInput(event: Event) {
    if (sendError) oneditdraft();
    // Read the field, not `draft`: this must not depend on bind:value running first.
    const text = (event.currentTarget as HTMLTextAreaElement).value;
    const now = Date.now();
    if (text.trim() && now - lastTypingAt >= TYPING_THROTTLE_MS) {
      lastTypingAt = now;
      ontyping();
    }
  }

  // "@par" at the start of the draft suggests full names; the server only reacts to "@bot ...".
  let mentions = $derived.by(() => {
    const query = group ? /^@(\S*)$/.exec(draft)?.[1] : undefined;
    if (query === undefined || !group) return [];
    const names = [
      ...group.bots,
      ...group.members.filter((m) => m !== currentUser),
    ];
    return names.filter((n) => n.toLowerCase().startsWith(query.toLowerCase()));
  });

  function pickMention(name: string) {
    draft = `@${name} `;
    inputEl?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Tab" && mentions.length) {
      event.preventDefault();
      pickMention(mentions[0]);
      return;
    }
    // Enter sends, Shift+Enter inserts a newline. While an IME is composing
    // (Burmese, CJK), Enter confirms the candidate and must not send.
    if (event.key !== "Enter" || event.shiftKey) return;
    if (event.isComposing || event.keyCode === 229) return;
    event.preventDefault();
    void submit();
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
                  {#each group.bots as bot (bot)}
                    <li>
                      <span class="av sm" style:background={avatarFill(bot)}
                        >{initial(bot)}</span
                      >
                      <span class="name">{bot}</span>
                      <span class="tag">bot</span>
                      {#if group.creator === currentUser}
                        <button
                          type="button"
                          class="ghost danger"
                          onclick={() => removeBot(group.id, bot)}
                          >Remove</button
                        >
                      {/if}
                    </li>
                  {/each}
                </ul>
                {#if group.creator === currentUser && group.bots.length < 3}
                  <form class="bot-form" onsubmit={addBot}>
                    <input placeholder="Bot name" bind:value={botName} />
                    <textarea
                      placeholder="Personality, e.g. You are a pirate. Talk like one."
                      maxlength="500"
                      bind:value={botPersonality}></textarea>
                    <button type="submit" class="ghost">Add bot</button>
                  </form>
                {/if}
              </div>
            {/if}
          </div>
          <button type="button" class="ghost" onclick={() => onleave(group.id)}
            >Leave</button
          >
          {#if group.creator === currentUser}
            <button
              type="button"
              class="ghost danger"
              onclick={() => ondelete(group)}>Delete</button
            >
          {/if}
        </div>
      {/if}
    </header>

    <div class="window" bind:this={windowEl}>
      <div class="feed">
        {#each rows as row (row.key)}
          {#if row.day}
            <p class="day"><span>{row.day}</span></p>
          {/if}
          {#if row.kind === "event"}
            <p class="event">
              <span>
                <strong>{row.event.user}</strong>
                {row.event.kind === "joined" ? "joined" : "left"} the group
                <time>{formatTime(row.event.timestamp)}</time>
              </span>
            </p>
          {:else}
            {@const { message, continued } = row}
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
                    {#if group?.bots.includes(message.sender)}<span class="you"
                        >bot</span
                      >{/if}
                    <time>{formatTime(message.timestamp)}</time>
                  </header>
                {/if}
                <p>{message.text}</p>
              </div>
            </article>
          {/if}
        {:else}
          <p class="empty-msg">No messages yet. Say hello.</p>
        {/each}
      </div>
    </div>

    <form class="composer" onsubmit={handleSubmit}>
      {#if sendError}
        <p class="send-error" role="alert">
          <Icon name="alert" size={14} />{sendError}
        </p>
      {:else if !peerOnline}
        <p class="send-note">{room.title} is offline.</p>
      {:else if typingLabel}
        <p class="send-note" role="status">{typingLabel}</p>
      {/if}
      {#if mentions.length}
        <div class="mentions">
          {#each mentions as name (name)}
            <button type="button" onclick={() => pickMention(name)}
              >@{name}</button
            >
          {/each}
        </div>
      {/if}
      <div class="box" class:disabled={!canSend}>
        <textarea
          bind:this={inputEl}
          bind:value={draft}
          rows="1"
          aria-label="Message"
          placeholder="Type a message…"
          disabled={!canSend}
          autocomplete="off"
          oninput={handleInput}
          onkeydown={handleKeydown}></textarea>
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

  .ghost.danger:hover {
    background: var(--err-soft);
    color: var(--err);
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

  .bot-form {
    display: grid;
    gap: 6px;
    padding: 6px;
    border-top: 1px solid var(--line-strong);
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

  /* Join/leave notices match the day divider's text weight, without its rules. */
  .day,
  .event {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    color: var(--faint);
    font-size: 12px;
    font-weight: 600;
  }

  .event {
    justify-content: center;
  }

  .day::before,
  .day::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--line);
  }

  .event time {
    font-weight: 400;
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

  .mentions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 0 2px 8px;
  }

  .mentions button {
    padding: 2px 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--raise);
    font-size: 12px;
    cursor: pointer;
  }

  .mentions button:first-child {
    border-color: var(--accent);
  }

  .box {
    display: flex;
    align-items: flex-end;
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

  .box textarea {
    flex: 1;
    min-width: 0;
    min-height: 32px;
    max-height: 144px;
    padding: 6px 0;
    border: 0;
    outline: none;
    resize: none;
    overflow-y: auto;
    background: none;
    color: var(--ink);
    line-height: 20px;
  }

  .box textarea::placeholder {
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
