<script>
  // Vertical bars, for distributions. bars: [{ label, value, tip }]
  import { fmt } from "../lib/stats.js";

  let { bars, height = 180, label = "", unit = "messages" } = $props();
  const max = $derived(Math.max(...bars.map((b) => b.value), 1));
</script>

<div class="cols" style:height="{height}px" role="img" aria-label={label}>
  {#each bars as b (b.label)}
    <div class="col" data-tip="{fmt(b.value)} {unit}" data-sub={b.tip ?? b.label}>
      <span class="v num" class:top={b.value === max}>{fmt(b.value)}</span>
      <i style:height="{(b.value / max) * 100}%"></i>
      <span class="l">{b.label}</span>
    </div>
  {/each}
</div>

<style>
  .cols { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: 8px; align-items: end; padding-top: 20px; }
  .col { height: 100%; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: 6px; min-width: 0; }
  i { display: block; width: 100%; max-width: 44px; min-height: 2px; background: var(--acc); border-radius: 4px 4px 0 0; }
  .col:hover i { filter: brightness(1.15); }
  .l { font-size: 12px; color: var(--t-muted); white-space: nowrap; }
  .v { font-size: 12px; color: var(--t-muted); visibility: hidden; }
  .v.top, .col:hover .v { visibility: visible; color: var(--t-head); }
</style>
