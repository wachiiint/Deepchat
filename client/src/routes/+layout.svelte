<script lang="ts">
  let { children } = $props();
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    rel="preconnect"
    href="https://fonts.gstatic.com"
    crossorigin="anonymous"
  />
  <link
    href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;9..144,700&family=Source+Sans+3:wght@400;500;600&display=swap"
    rel="stylesheet"
  />
</svelte:head>

{@render children()}

<style>
  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    margin: 0;
    min-height: 100%;
    height: 100%;
  }

  :global(body) {
    font-family: "Source Sans 3", "Segoe UI", sans-serif;
    color: #1f1e1b;
    background: #f3f0ea;
  }

  :global(#svelte) {
    height: 100%;
  }
</style>
