<script lang="ts">
  import "@fontsource-variable/inter";

  let { children } = $props();
</script>

{@render children()}

<style>
  :global(:root) {
    --bg: #0e1a33;
    --side: #0a1428;
    --raise: #15264a;
    --raise-hover: #1b2e57;
    --line: #1f3359;
    --line-strong: #2b4372;
    --ink: #e8eef8;
    --text: #d5deec;
    --mute: #a3b3cd;
    --faint: #7f92b1;
    --accent: #4fd8eb;
    --accent-hover: #80e5f2;
    --accent-ink: #071224;
    --accent-soft: rgb(79 216 235 / 0.14);
    --ok: #3ddc97;
    --err: #ff7b7b;
    --err-soft: rgb(255 123 123 / 0.12);
    --radius-sm: 6px;
    --radius-md: 8px;
    --radius-lg: 10px;

    color-scheme: dark;
    accent-color: var(--accent);
  }

  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }

  :global(body) {
    font:
      14px/1.5 "Inter Variable",
      system-ui,
      sans-serif;
    font-feature-settings: "cv11", "ss01";
    color: var(--ink);
    background: var(--bg);
    -webkit-font-smoothing: antialiased;
  }

  :global(::selection) {
    background: rgb(79 216 235 / 0.3);
    color: var(--ink);
  }

  :global(button),
  :global(input) {
    font: inherit;
    color: inherit;
  }

  :global(button) {
    cursor: pointer;
  }

  :global(button:disabled) {
    cursor: not-allowed;
  }

  :global(:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  :global(*) {
    scrollbar-width: thin;
    scrollbar-color: var(--line-strong) transparent;
  }

  :global(h1),
  :global(h2),
  :global(h3),
  :global(p),
  :global(ul) {
    margin: 0;
  }
</style>
