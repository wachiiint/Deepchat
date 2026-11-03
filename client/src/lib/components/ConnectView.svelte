<script lang="ts">
  type ConnectPayload = {
    host: string;
    port: number;
    name: string;
  };

  let {
    busy = false,
    error = "",
    onconnect,
  }: {
    busy?: boolean;
    error?: string;
    onconnect: (payload: ConnectPayload) => void | Promise<void>;
  } = $props();

  let host = $state("127.0.0.1");
  let port = $state("7878");
  let name = $state("");

  async function handleSubmit(event: Event) {
    event.preventDefault();
    const parsedPort = Number(port);
    if (
      !Number.isInteger(parsedPort) ||
      parsedPort <= 0 ||
      parsedPort > 65535
    ) {
      return;
    }
    await onconnect({ host: host.trim(), port: parsedPort, name: name.trim() });
  }
</script>

<section class="connect">
  <div class="card">
    <p class="brand">Deepchat</p>
    <h1>Join the chat</h1>
    <p class="hint">
      Connect to your Deepchat server to chat with other users.
    </p>

    <form onsubmit={handleSubmit}>
      <label>
        Display name
        <input
          bind:value={name}
          placeholder="your unique name"
          required
          disabled={busy}
        />
      </label>
      <div class="row">
        <label>
          Host
          <input
            bind:value={host}
            placeholder="127.0.0.1"
            required
            disabled={busy}
          />
        </label>
        <label class="port">
          Port
          <input
            bind:value={port}
            type="number"
            min="1"
            max="65535"
            required
            disabled={busy}
          />
        </label>
      </div>
      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}
      <button type="submit" disabled={busy || !name.trim()}>
        {busy ? "Connecting…" : "Connect"}
      </button>
    </form>
  </div>
</section>

<style>
  .connect {
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 2rem;
    background:
      radial-gradient(ellipse at top left, #d8ebe4 0%, transparent 55%),
      radial-gradient(ellipse at bottom right, #e6dfd0 0%, transparent 50%),
      #f3f0ea;
  }

  .card {
    width: min(420px, 100%);
    padding: 2rem;
    border: 1px solid #cfc8bc;
    background: rgba(255, 252, 247, 0.92);
    box-shadow: 0 18px 40px rgba(40, 35, 25, 0.08);
  }

  .brand {
    margin: 0 0 0.35rem;
    font-family: "Fraunces", "Iowan Old Style", Georgia, serif;
    font-size: 1.75rem;
    letter-spacing: -0.02em;
    color: #1f3d34;
  }

  h1 {
    margin: 0 0 0.4rem;
    font-size: 1.15rem;
    font-weight: 600;
    color: #2a2a28;
  }

  .hint {
    margin: 0 0 1.4rem;
    color: #6a655c;
    font-size: 0.9rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: #4a463e;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 7rem;
    gap: 0.75rem;
  }

  input {
    border: 1px solid #cfc8bc;
    background: #fff;
    padding: 0.65rem 0.75rem;
    font: inherit;
    color: #1f1e1b;
  }

  input:focus {
    outline: 2px solid #2f6f5e;
    outline-offset: 1px;
  }

  button {
    margin-top: 0.35rem;
    border: none;
    background: #1f3d34;
    color: #f7f3eb;
    padding: 0.75rem 1rem;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  button:not(:disabled):hover {
    background: #2f6f5e;
  }

  .error {
    margin: 0;
    color: #9b2c2c;
    font-size: 0.9rem;
  }
</style>
