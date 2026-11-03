<script lang="ts">
  import type { Group } from "$lib/types";
  import { avatarFill, initial } from "$lib/avatar";
  import Icon from "./Icon.svelte";

  let {
    group,
    currentUser = "",
    busy = false,
    joining = false,
    onjoin,
    ondelete,
  }: {
    group: Group;
    currentUser?: string;
    busy?: boolean;
    joining?: boolean;
    onjoin: (groupId: string) => void | Promise<void>;
    ondelete: (group: Group) => void;
  } = $props();
</script>

<section class="preview" aria-labelledby="preview-title">
  <header class="head">
    <h1 id="preview-title">
      <span class="glyph"><Icon name="hash" /></span>
      <span class="title">{group.name}</span>
    </h1>
    <p class="meta">Created by {group.creator}</p>
    {#if group.creator === currentUser}
      <button type="button" class="delete" onclick={() => ondelete(group)}
        >Delete</button
      >
    {/if}
  </header>

  <div class="content">
    <p class="lead">Join this group before opening the chat room.</p>
    <button
      type="button"
      class="join"
      disabled={busy || joining}
      onclick={() => onjoin(group.id)}
    >
      {joining ? "Joining…" : `Join ${group.name}`}
    </button>

    <h2>
      {group.members.length} member{group.members.length === 1 ? "" : "s"}
    </h2>
    <ul>
      {#each group.members as member (member)}
        <li>
          <span
            class="av"
            style:background={avatarFill(member)}
            aria-hidden="true">{initial(member)}</span
          >
          <span class="name">{member}</span>
          {#if member === group.creator}<span class="tag">creator</span>{/if}
        </li>
      {/each}
    </ul>
  </div>
</section>

<style>
  .preview {
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 0;
  }

  .head {
    height: 56px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 24px;
    border-bottom: 1px solid var(--line);
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
  }

  .delete {
    margin-left: auto;
    flex: none;
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
    font-size: 13px;
    font-weight: 500;
  }

  .delete:hover {
    background: var(--err-soft);
    color: var(--err);
  }

  .content {
    overflow-y: auto;
    padding: 32px 24px;
  }

  .content > * {
    max-width: 420px;
  }

  .lead {
    color: var(--mute);
  }

  .join {
    margin-top: 16px;
    height: 40px;
    padding: 0 20px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 650;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .join:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .join:disabled {
    background: var(--raise);
    color: var(--faint);
  }

  h2 {
    margin-top: 40px;
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--faint);
  }

  ul {
    list-style: none;
    padding: 0;
    border-top: 1px solid var(--line);
  }

  li {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    border-bottom: 1px solid var(--line);
  }

  .av {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    display: grid;
    place-items: center;
    color: #fff;
    font-weight: 700;
    font-size: 11px;
    flex: none;
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    font-size: 12px;
    color: var(--faint);
  }
</style>
