<script>
  import { fmt } from "../lib/stats.js";

  let { alive, deleted, lost } = $props();
  const sent = $derived(alive + deleted + lost);
  const pct = (n) => `${((n / Math.max(sent, 1)) * 100).toFixed(1)}%`;
</script>

<div class="bar" role="img" aria-label="{fmt(alive)} still there, {fmt(deleted)} deleted, {fmt(lost)} deleted server">
  <i class="alive" style:flex={alive} data-tip="{fmt(alive)} still there" data-sub="{pct(alive)} of sent"></i>
  {#if deleted}<i class="deleted" style:flex={deleted} data-tip="{fmt(deleted)} deleted" data-sub={pct(deleted)}></i>{/if}
  {#if lost}<i class="lost" style:flex={lost} data-tip="{fmt(lost)} deleted server" data-sub={pct(lost)}></i>{/if}
</div>

<style>
  .bar { display: flex; gap: 2px; height: 20px; border-radius: 6px; overflow: hidden; background: var(--sunk); }
  i { display: block; height: 100%; min-width: 2px; }
  .alive { background: var(--acc); }
  .deleted { background: var(--t-muted); }
  .lost { background: repeating-linear-gradient(135deg, var(--t-faint) 0 2px, transparent 2px 5px); }
</style>
