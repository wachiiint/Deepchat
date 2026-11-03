<script lang="ts">
  let {
    title,
    message,
    confirmLabel,
    busy = false,
    onconfirm,
    oncancel,
  }: {
    title: string;
    message: string;
    confirmLabel: string;
    busy?: boolean;
    onconfirm: () => void | Promise<void>;
    oncancel: () => void;
  } = $props();

  let dialogEl: HTMLDialogElement | undefined = $state();
  let cancelEl: HTMLButtonElement | undefined = $state();

  $effect(() => {
    if (!dialogEl || dialogEl.open) return;
    dialogEl.showModal();
    // The destructive action is never the default focus.
    cancelEl?.focus();
  });

  function handleClick(event: MouseEvent) {
    // A click on the backdrop lands on the dialog element itself.
    if (event.target === dialogEl) oncancel();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<dialog
  bind:this={dialogEl}
  aria-labelledby="confirm-title"
  aria-describedby="confirm-message"
  onclick={handleClick}
  oncancel={(event) => {
    event.preventDefault();
    oncancel();
  }}
>
  <h2 id="confirm-title">{title}</h2>
  <p id="confirm-message">{message}</p>
  <div class="buttons">
    <button type="button" class="cancel" bind:this={cancelEl} onclick={oncancel}
      >Cancel</button
    >
    <button type="button" class="danger" disabled={busy} onclick={onconfirm}
      >{confirmLabel}</button
    >
  </div>
</dialog>

<style>
  dialog {
    margin: auto;
    width: min(400px, calc(100vw - 32px));
    padding: 20px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--raise);
    color: var(--text);
    box-shadow: 0 12px 32px rgb(3 8 20 / 0.5);
  }

  dialog::backdrop {
    background: rgb(3 8 20 / 0.6);
  }

  h2 {
    font-size: 16px;
    font-weight: 650;
    color: var(--ink);
    overflow-wrap: anywhere;
  }

  p {
    margin-top: 8px;
    color: var(--mute);
    overflow-wrap: anywhere;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }

  button {
    height: 32px;
    padding: 0 14px;
    border: 0;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
  }

  .cancel {
    background: none;
    color: var(--mute);
  }

  .cancel:hover {
    background: var(--raise-hover);
    color: var(--ink);
  }

  .danger {
    background: var(--err);
    color: var(--accent-ink);
  }

  .danger:hover:not(:disabled) {
    background: #ff9a9a;
  }

  .danger:disabled {
    background: var(--raise-hover);
    color: var(--faint);
  }
</style>
