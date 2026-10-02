<script>
  import { FILLED, ICONS } from "../lib/icons.js";

  let { name, size = 20, stroke = 2 } = $props();
  const filled = $derived(FILLED.has(name));
</script>

<svg
  class="icon"
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill={filled ? "currentColor" : "none"}
  stroke={filled ? "none" : "currentColor"}
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {@html ICONS[name]}
</svg>

<style>
  .icon { display: block; flex: none; }
</style>
