<script lang="ts">
  let {
    users = [],
    currentUser = "",
    selectedPeer = null,
    unread = {},
    onselect,
  }: {
    users?: string[];
    currentUser?: string;
    selectedPeer?: string | null;
    unread?: Record<string, number>;
    onselect: (peer: string) => void | Promise<void>;
  } = $props();
</script>

<section aria-labelledby="users-heading">
  <h2 id="users-heading">
    <span>Active users</span>
    <span class="total">{users.length}</span>
  </h2>
  <ul>
    {#each users as user (user)}
      {@const me = user === currentUser}
      {@const count = unread[user] ?? 0}
      <li>
        <button
          type="button"
          class="item"
          class:on={selectedPeer === user}
          class:unread={count > 0}
          class:me
          disabled={me}
          aria-current={selectedPeer === user ? "true" : undefined}
          onclick={() => onselect(user)}
        >
          <span class="dot" aria-hidden="true"></span>
          <span class="name">{user}</span>
          {#if me}
            <span class="you">you</span>
          {:else if count > 0}
            <span class="count" aria-label={`${count} unread`}
              >{count > 99 ? "99+" : count}</span
            >
          {/if}
        </button>
      </li>
    {:else}
      <li class="empty">No users connected</li>
    {/each}
  </ul>
</section>

<style>
  section {
    padding: 16px 8px 8px;
  }

  h2 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 12px 0 8px;
    margin-bottom: 2px;
    font-size: 12px;
    font-weight: 600;
    color: var(--faint);
  }

  .total {
    font-variant-numeric: tabular-nums;
  }

  ul {
    list-style: none;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .item {
    width: 100%;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px 0 12px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
    text-align: left;
  }

  .item:hover:not(:disabled) {
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

  .item.me {
    cursor: default;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    flex: none;
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .you {
    font-size: 12px;
    color: var(--faint);
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

  .empty {
    padding: 4px 12px;
    color: var(--faint);
    font-size: 13px;
  }
</style>
