<script lang="ts">
  export type IconName =
    "hash" | "plus" | "users" | "user" | "logout" | "send" | "close" | "alert";

  let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "hash"}
    <path d="M6 2.5 4.5 13.5M11.5 2.5 10 13.5M2.5 5.5h11M2 10.5h11" />
  {:else if name === "plus"}
    <path d="M8 3v10M3 8h10" />
  {:else if name === "users"}
    <circle cx="6" cy="5.5" r="2.5" />
    <path
      d="M1.5 13c.6-2.3 2.3-3.5 4.5-3.5s3.9 1.2 4.5 3.5M10.5 3.2a2.5 2.5 0 0 1 0 4.6M12.5 9.8c1 .6 1.7 1.6 2 3.2"
    />
  {:else if name === "user"}
    <circle cx="8" cy="5.5" r="2.75" />
    <path d="M2.75 13.5c.7-2.6 2.7-4 5.25-4s4.55 1.4 5.25 4" />
  {:else if name === "logout"}
    <path
      d="M10 3h2.5a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H10M6.5 11 3.5 8l3-3M3.5 8h7"
    />
  {:else if name === "send"}
    <path d="M8 13V3M3.5 7.5 8 3l4.5 4.5" />
  {:else if name === "close"}
    <path d="M4 4l8 8M12 4l-8 8" />
  {:else if name === "alert"}
    <circle cx="8" cy="8" r="6" />
    <path d="M8 4.75v3.75M8 11.25h.01" />
  {/if}
</svg>

<style>
  svg {
    display: block;
    flex: none;
  }
</style>
