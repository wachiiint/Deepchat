<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "./Icon.svelte";

  export type ConnectPayload = {
    host: string;
    port: number;
    name: string;
  };

  let {
    busy = false,
    error = "",
    notice = "",
    initial,
    onconnect,
  }: {
    busy?: boolean;
    error?: string;
    notice?: string;
    initial?: ConnectPayload | null;
    onconnect: (payload: ConnectPayload) => void | Promise<void>;
  } = $props();

  // Prefill once from the last attempt; later edits belong to the user.
  const seed = untrack(() => initial);
  let host = $state(seed?.host ?? "127.0.0.1");
  let port = $state(String(seed?.port ?? 7878));
  let name = $state(seed?.name ?? "");

  const portValid = $derived.by(() => {
    const n = Number(port);
    return Number.isInteger(n) && n > 0 && n <= 65535;
  });
  // The backend reports name conflicts in prose; anything else is about reaching the server.
  const nameError = $derived(/display name/i.test(error) ? error : "");
  const serverError = $derived(error && !nameError ? error : "");

  async function handleSubmit(event: Event) {
    event.preventDefault();
    if (!portValid || !name.trim() || !host.trim()) return;
    await onconnect({
      host: host.trim(),
      port: Number(port),
      name: name.trim(),
    });
  }
</script>

<main class="connect">
  <p class="brand">Deepchat</p>
  <form onsubmit={handleSubmit} novalidate>
    <h1>Join the chat</h1>
    <p class="hint">
      Connect to your Deepchat server to chat with other users.
    </p>

    {#if notice}
      <p class="banner" role="alert">
        <Icon name="alert" />
        <span>{notice}</span>
      </p>
    {/if}

    <div class="field">
      <label for="connect-name">Display name</label>
      <input
        id="connect-name"
        bind:value={name}
        placeholder="your unique name"
        autocomplete="off"
        spellcheck="false"
        required
        disabled={busy}
        aria-invalid={nameError ? "true" : undefined}
        aria-describedby={nameError ? "connect-name-error" : undefined}
      />
      {#if nameError}
        <p class="error" id="connect-name-error" role="alert">{nameError}</p>
      {/if}
    </div>

    <div class="row">
      <div class="field">
        <label for="connect-host">Host</label>
        <input
          id="connect-host"
          bind:value={host}
          placeholder="127.0.0.1"
          autocomplete="off"
          spellcheck="false"
          required
          disabled={busy}
          aria-invalid={serverError ? "true" : undefined}
        />
      </div>
      <div class="field port">
        <label for="connect-port">Port</label>
        <input
          id="connect-port"
          bind:value={port}
          inputmode="numeric"
          autocomplete="off"
          required
          disabled={busy}
          aria-invalid={serverError || !portValid ? "true" : undefined}
        />
      </div>
    </div>
    {#if serverError}
      <p class="error" role="alert">{serverError}</p>
    {:else if !portValid && port !== ""}
      <p class="error">Port must be a number from 1 to 65535.</p>
    {/if}

    <button
      type="submit"
      class="submit"
      disabled={busy || !name.trim() || !host.trim() || !portValid}
    >
      {busy ? "Connecting…" : "Connect"}
    </button>
  </form>
</main>

<style>
  .connect {
    position: relative;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    /* Fixed top offset: banners and errors grow the form downward without moving the heading. */
    padding: max(32px, 22vh) 32px 32px;
    overflow-y: auto;
    background: var(--bg);
  }

  form {
    width: min(360px, 100%);
    display: flex;
    flex-direction: column;
  }

  .brand {
    position: absolute;
    top: 16px;
    left: 16px;
    font-size: 16px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  h1 {
    font-size: 24px;
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.2;
  }

  .hint {
    color: var(--mute);
    margin: 6px 0 28px;
  }

  .banner {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 10px 12px;
    margin-bottom: 20px;
    border-radius: var(--radius-md);
    background: var(--err-soft);
    color: var(--ink);
    font-size: 13px;
  }

  .banner :global(svg) {
    color: var(--err);
    margin-top: 2px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
    min-width: 0;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 96px;
    gap: 8px;
  }

  .row .field {
    margin-bottom: 0;
  }

  label {
    font-size: 13px;
    font-weight: 600;
  }

  input {
    height: 40px;
    padding: 0 12px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    background: var(--raise);
    color: var(--ink);
    transition:
      border-color 120ms ease,
      box-shadow 120ms ease;
  }

  input::placeholder {
    color: var(--faint);
  }

  input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  input[aria-invalid="true"] {
    border-color: var(--err);
  }

  input:disabled {
    opacity: 0.6;
  }

  .error {
    color: var(--err);
    font-size: 12px;
    line-height: 1.4;
    margin-top: 6px;
  }

  .field .error {
    margin-top: 0;
  }

  .submit {
    height: 40px;
    margin-top: 24px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 650;
    transition: background-color 120ms ease;
  }

  .submit:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .submit:disabled {
    background: var(--raise);
    color: var(--faint);
  }
</style>
