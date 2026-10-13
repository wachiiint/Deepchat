<script lang="ts">
  import type { ActiveRoom, Message } from "$lib/types";

  let {
    room = null,
    messages = [],
    currentUser = "",
    busy = false,
    onsend,
    onsimulate,
  }: {
    room?: ActiveRoom | null;
    messages?: Message[];
    currentUser?: string;
    busy?: boolean;
    onsend: (text: string) => void | Promise<void>;
    onsimulate: () => void | Promise<void>;
  } = $props();

  let draft = $state("");
  let windowEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    messages;
    if (windowEl) {
      windowEl.scrollTop = windowEl.scrollHeight;
    }
  });

  async function handleSend(event: Event) {
    event.preventDefault();
    const text = draft.trim();
    if (!text || !room) return;
    await onsend(text);
    draft = "";
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<section class="room">
  {#if !room}
    <div class="empty-state">
      <p class="brand">Deepchat</p>
      <p>Select a user or group to open a chat room.</p>
    </div>
  {:else}
    <header>
      <div>
        <p class="kind">{room.kind === "private" ? "Private" : "Group"}</p>
        <h2>{room.title}</h2>
      </div>
      <button type="button" class="sim" onclick={() => onsimulate()} disabled={busy}>
        Simulate incoming
      </button>
    </header>

    <div class="window" bind:this={windowEl}>
      {#each messages as message (message.id)}
        <article class="bubble" class:mine={message.sender === currentUser}>
          <header class="meta">
            <strong>{message.sender}</strong>
            <time>{formatTime(message.timestamp)}</time>
          </header>
          <p>{message.text}</p>
        </article>
      {:else}
        <p class="empty-msg">No messages yet. Say hello.</p>
      {/each}
    </div>

    <form class="composer" onsubmit={handleSend}>
      <input
        bind:value={draft}
        placeholder="Type a message…"
        disabled={busy}
        autocomplete="off"
      />
      <button type="submit" disabled={busy || !draft.trim()}>Send</button>
    </form>
  {/if}
</section>

<style>
  .room {
    display: grid;
    grid-template-rows: auto 1fr auto;
    min-height: 0;
    height: 100%;
    background: #fbf8f2;
  }

  .empty-state {
    grid-row: 1 / -1;
    display: grid;
    place-content: center;
    gap: 0.4rem;
    text-align: center;
    color: #6a655c;
    padding: 2rem;
  }

  .brand {
    margin: 0;
    font-family: "Fraunces", "Iowan Old Style", Georgia, serif;
    font-size: 2rem;
    color: #1f3d34;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.9rem 1.1rem;
    border-bottom: 1px solid #ddd6ca;
    background: rgba(255, 252, 247, 0.9);
  }

  .kind {
    margin: 0;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #8a8376;
  }

  h2 {
    margin: 0.1rem 0 0;
    font-size: 1.1rem;
  }

  .sim {
    border: 1px solid #cfc8bc;
    background: #fff;
    padding: 0.45rem 0.7rem;
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .sim:hover:not(:disabled) {
    border-color: #2f6f5e;
  }

  .sim:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .window {
    overflow: auto;
    padding: 1rem 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .bubble {
    max-width: min(32rem, 85%);
    align-self: flex-start;
    padding: 0.65rem 0.8rem;
    background: #fff;
    border: 1px solid #ddd6ca;
  }

  .bubble.mine {
    align-self: flex-end;
    background: #d8ebe4;
    border-color: #b7d4c9;
  }

  .meta {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.25rem;
    font-size: 0.8rem;
    color: #6a655c;
    border: none;
    padding: 0;
    background: transparent;
  }

  .bubble p {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .empty-msg {
    margin: auto;
    color: #8a8376;
  }

  .composer {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem;
    padding: 0.85rem 1.1rem;
    border-top: 1px solid #ddd6ca;
    background: #fffefb;
  }

  .composer input {
    border: 1px solid #cfc8bc;
    padding: 0.7rem 0.8rem;
    font: inherit;
    min-width: 0;
  }

  .composer button {
    border: none;
    background: #1f3d34;
    color: #f7f3eb;
    padding: 0.7rem 1.1rem;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }

  .composer button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .composer button:not(:disabled):hover {
    background: #2f6f5e;
  }
</style>
