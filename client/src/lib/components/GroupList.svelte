<script lang="ts">
  import type { Group } from "$lib/types";
  import Icon from "./Icon.svelte";

  let {
    groups = [],
    currentUser = "",
    selectedGroupId = null,
    unread = {},
    busy = false,
    creating = false,
    createError = "",
    createOpen = $bindable(false),
    onselect,
    onjoin,
    oncreate,
    oncreatecancel,
  }: {
    groups?: Group[];
    currentUser?: string;
    selectedGroupId?: string | null;
    unread?: Record<string, number>;
    busy?: boolean;
    creating?: boolean;
    createError?: string;
    createOpen?: boolean;
    onselect: (group: Group) => void | Promise<void>;
    onjoin: (groupId: string) => void | Promise<void>;
    oncreate: (name: string) => void | Promise<void>;
    oncreatecancel: () => void;
  } = $props();

  let newGroupName = $state("");
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (createOpen) inputEl?.focus();
    else newGroupName = "";
  });

  // A rejected name stays selected so it can be retyped at once.
  $effect(() => {
    if (createError) inputEl?.select();
  });

  async function handleCreate(event: Event) {
    event.preventDefault();
    const name = newGroupName.trim();
    if (!name || creating) return;
    await oncreate(name);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      oncreatecancel();
    }
  }
</script>

<section aria-labelledby="groups-heading">
  <h2 id="groups-heading">
    <span>Groups</span>
    <button
      type="button"
      class="icon"
      aria-label="Create group"
      title="Create group"
      aria-expanded={createOpen}
      onclick={() => (createOpen ? oncreatecancel() : (createOpen = true))}
    >
      <Icon name={createOpen ? "close" : "plus"} />
    </button>
  </h2>

  {#if createOpen}
    <form class="create" onsubmit={handleCreate}>
      <input
        bind:this={inputEl}
        bind:value={newGroupName}
        aria-label="New group name"
        placeholder="New group name"
        autocomplete="off"
        spellcheck="false"
        disabled={creating}
        aria-invalid={createError ? "true" : undefined}
        aria-describedby={createError ? "create-error" : undefined}
        onkeydown={handleKeydown}
      />
      <button type="submit" disabled={creating || !newGroupName.trim()}>
        {creating ? "Creating…" : "Create"}
      </button>
      {#if createError}
        <p class="error" id="create-error" role="alert">{createError}</p>
      {/if}
    </form>
  {/if}

  <ul>
    {#each groups as group (group.id)}
      {@const isMember = group.members.includes(currentUser)}
      {@const count = unread[group.id] ?? 0}
      <li class:member={isMember}>
        <button
          type="button"
          class="item"
          class:on={selectedGroupId === group.id}
          class:unread={count > 0}
          aria-current={selectedGroupId === group.id ? "true" : undefined}
          title={`${group.members.length} member${group.members.length === 1 ? "" : "s"}: ${group.members.join(", ")}`}
          onclick={() => onselect(group)}
        >
          <Icon name="hash" />
          <span class="name">{group.name}</span>
          {#if count > 0}
            <span class="count" aria-label={`${count} unread`}
              >{count > 99 ? "99+" : count}</span
            >
          {/if}
        </button>
        {#if !isMember}
          <button
            type="button"
            class="join"
            disabled={busy}
            onclick={() => onjoin(group.id)}
            aria-label={`Join ${group.name}`}
          >
            Join
          </button>
        {/if}
      </li>
    {:else}
      <li class="empty">No groups yet</li>
    {/each}
  </ul>
</section>

<style>
  section {
    padding: 16px 8px 4px;
  }

  h2 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 4px 0 8px;
    margin-bottom: 2px;
    font-size: 12px;
    font-weight: 600;
    color: var(--faint);
  }

  .icon {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
  }

  .icon:hover {
    background: var(--raise);
    color: var(--ink);
  }

  .create {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 6px;
    padding: 4px 0 8px;
  }

  .create input {
    min-width: 0;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--raise);
    color: var(--ink);
  }

  .create input::placeholder {
    color: var(--faint);
  }

  .create input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .create input[aria-invalid="true"] {
    border-color: var(--err);
  }

  .create button {
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 13px;
    font-weight: 650;
  }

  .create button:disabled {
    background: var(--raise);
    color: var(--faint);
  }

  .error {
    grid-column: 1 / -1;
    color: var(--err);
    font-size: 12px;
    line-height: 1.4;
    padding: 0 2px;
  }

  ul {
    list-style: none;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  li {
    position: relative;
  }

  .item {
    width: 100%;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--faint);
    text-align: left;
  }

  li.member .item {
    color: var(--mute);
  }

  li:not(.member) .item {
    padding-right: 64px;
  }

  .item:hover {
    background: var(--raise);
    color: var(--ink);
  }

  .item.unread {
    color: var(--ink);
    font-weight: 600;
  }

  .item.on,
  .item.on:hover {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    border-radius: 10px;
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 11px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    display: grid;
    place-items: center;
  }

  .join {
    position: absolute;
    right: 4px;
    top: 4px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--side);
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
  }

  .join:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .join:disabled {
    color: var(--faint);
  }

  .empty {
    padding: 4px 8px;
    color: var(--faint);
    font-size: 13px;
  }
</style>
