<script lang="ts">
  import type { Group } from "$lib/types";

  let {
    groups = [],
    currentUser = "",
    selectedGroupId = null,
    onselect,
    onjoin,
    oncreate,
    onleave,
    busy = false,
  }: {
    groups?: Group[];
    busy?: boolean;
    onleave: (groupId: string) => void | Promise<void>;
    currentUser?: string;
    selectedGroupId?: string | null;
    onselect: (group: Group) => void | Promise<void>;
    onjoin: (groupId: string) => void | Promise<void>;
    oncreate: (name: string) => void | Promise<void>;
  } = $props();

  let newGroupName = $state("");

  async function handleCreate(event: Event) {
    event.preventDefault();
    const name = newGroupName.trim();
    if (!name) return;
    await oncreate(name);
    newGroupName = "";
  }
</script>

<section class="panel">
  <header>
    <h2>Groups</h2>
    <span class="count">{groups.length}</span>
  </header>

  <form class="create" onsubmit={handleCreate}>
    <input bind:value={newGroupName} aria-label="New group name" placeholder="New group name" disabled={busy} />
    <button type="submit" disabled={busy || !newGroupName.trim()}>Create</button>
  </form>

  <ul>
    {#each groups as group (group.id)}
      {@const isMember = group.members.includes(currentUser)}
      <li>
        <button
          type="button"
          class="group"
          class:selected={selectedGroupId === group.id}
          disabled={busy || !isMember}
          onclick={() => onselect(group)}
        >
          <span class="title-row">
            <span class="name">{group.name}</span>
            {#if isMember}
              <span class="badge">joined</span>
            {/if}
          </span>
          <span class="members">Created by {group.creator}</span>
          <span class="members">{group.members.join(", ")}</span>
        </button>
        {#if !isMember}
          <button type="button" class="join" disabled={busy} onclick={() => onjoin(group.id)}>
            Join
          </button>
        {:else}
          <button type="button" class="join" disabled={busy} onclick={() => onleave(group.id)}>Leave</button>
        {/if}
      </li>
    {:else}
      <li class="empty">No groups yet</li>
    {/each}
  </ul>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-top: 1px solid #ddd6ca;
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.85rem 1rem 0.5rem;
  }

  h2 {
    margin: 0;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #6a655c;
  }

  .count {
    font-size: 0.8rem;
    color: #8a8376;
  }

  .create {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.4rem;
    padding: 0 0.75rem 0.75rem;
  }

  input {
    border: 1px solid #cfc8bc;
    background: #fff;
    padding: 0.45rem 0.55rem;
    font: inherit;
    min-width: 0;
  }

  .create button {
    border: none;
    background: #1f3d34;
    color: #f7f3eb;
    padding: 0.45rem 0.7rem;
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .create button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0 0.5rem 0.75rem;
    overflow: auto;
  }

  li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.35rem;
    align-items: stretch;
    margin-bottom: 0.35rem;
  }

  .group {
    border: none;
    background: transparent;
    padding: 0.55rem 0.65rem;
    text-align: left;
    font: inherit;
    color: #1f1e1b;
    cursor: pointer;
    min-width: 0;
    overflow-wrap: anywhere;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .group:disabled { cursor: default; }

  .group:hover:not(:disabled) {
    background: #ebe6dc;
  }

  .group.selected {
    background: #d8ebe4;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .name {
    font-weight: 600;
  }

  .members {
    font-size: 0.8rem;
    color: #6a655c;
  }

  .badge {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #2f6f5e;
  }

  .join {
    border: 1px solid #cfc8bc;
    background: #fff;
    padding: 0.35rem 0.55rem;
    font: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    align-self: center;
  }

  .join:hover {
    border-color: #2f6f5e;
    color: #1f3d34;
  }

  .empty {
    padding: 0.75rem;
    color: #8a8376;
    font-size: 0.9rem;
    grid-column: 1 / -1;
  }
</style>
