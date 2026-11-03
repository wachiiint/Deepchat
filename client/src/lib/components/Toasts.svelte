<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { prefersReducedMotion } from "svelte/motion";
  import { toasts } from "$lib/toasts.svelte";
  import Icon from "./Icon.svelte";

  const motion = $derived({
    x: prefersReducedMotion.current ? 0 : 24,
    duration: prefersReducedMotion.current ? 0 : 240,
    easing: cubicOut,
  });
</script>

<div class="stack" aria-live="assertive">
  {#each toasts.items as toast (toast.id)}
    <div
      class="toast"
      role="alert"
      in:fly={motion}
      out:fly={{ ...motion, duration: motion.duration * 0.6 }}
      onmouseenter={() => toasts.pause(toast.id)}
      onmouseleave={() => toasts.resume(toast.id)}
    >
      <Icon name="alert" />
      <p>{toast.text}</p>
      <button
        type="button"
        aria-label="Dismiss"
        onclick={() => toasts.dismiss(toast.id)}
      >
        <Icon name="close" size={14} />
      </button>
    </div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    top: 68px;
    right: 16px;
    z-index: 50;
    width: min(360px, calc(100vw - 32px));
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 8px 12px 12px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--raise);
    box-shadow: 0 12px 32px rgb(3 8 20 / 0.5);
    color: var(--ink);
    font-size: 13px;
    line-height: 1.45;
  }

  .toast > :global(svg) {
    color: var(--err);
    margin-top: 1px;
  }

  p {
    flex: 1;
    min-width: 0;
  }

  button {
    width: 24px;
    height: 24px;
    margin-top: -3px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
  }

  button:hover {
    background: var(--raise-hover);
    color: var(--ink);
  }
</style>
