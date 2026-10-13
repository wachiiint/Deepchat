<script lang="ts">
  let {
    users = [],
    currentUser = "",
    selectedPeer = null,
    onselect,
  }: {
    users?: string[];
    currentUser?: string;
    selectedPeer?: string | null;
    onselect: (peer: string) => void | Promise<void>;
  } = $props();
</script>

<section class="panel">
  <header>
    <h2>Active users</h2>
    <span class="count">{users.length}</span>
  </header>
  <ul>
    {#each users as user (user)}
      <li>
        <button
          type="button"
          class:selected={selectedPeer === user}
          class:me={user === currentUser}
          disabled={user === currentUser}
          onclick={() => onselect(user)}
        >
          <span class="name">{user}</span>
          {#if user === currentUser}
            <span class="badge">you</span>
          {/if}
        </button>
      </li>
    {:else}
      <li class="empty">No users connected</li>
    {/each}
  </ul>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
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

  ul {
    list-style: none;
    margin: 0;
    padding: 0 0.5rem 0.75rem;
    overflow: auto;
  }

  button {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    border: none;
    background: transparent;
    padding: 0.55rem 0.65rem;
    text-align: left;
    font: inherit;
    color: #1f1e1b;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: #ebe6dc;
  }

  button.selected {
    background: #d8ebe4;
    color: #1f3d34;
  }

  button.me {
    cursor: default;
    opacity: 0.85;
  }

  .badge {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #2f6f5e;
  }

  .empty {
    padding: 0.75rem;
    color: #8a8376;
    font-size: 0.9rem;
  }
</style>
