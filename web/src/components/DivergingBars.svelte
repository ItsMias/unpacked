<script>
  // Bars above and below a zero line, per period: gains up, losses down.
  //   points: [{ label, up, down }]
  import { fmt } from "../lib/stats.js";

  let { points, upLabel, downLabel, upColour = "var(--acc)", downColour = "var(--t-muted)", height = 200 } = $props();

  const max = $derived(Math.max(1, ...points.flatMap((p) => [p.up, p.down])));
  const half = $derived((height - 24) / 2);
  // Narrow columns (many years on a phone) get two-digit years: '15.
  let width = $state(600);
  const short = $derived(width / Math.max(points.length, 1) < 36);
  const label = (l) => (short && /^\d{4}$/.test(l) ? `'${l.slice(2)}` : l);
</script>

<div class="wrap">
  <div class="key">
    <span><i style:background={upColour}></i>{upLabel}</span>
    <span><i style:background={downColour}></i>{downLabel}</span>
  </div>
  <div class="bars" style:height="{height}px" style:--n={points.length} bind:clientWidth={width}>
    <span class="zero" style:top="{half}px"></span>
    {#each points as p (p.label)}
      <div class="col" data-tip={p.label} data-sub="{fmt(p.up)} {upLabel.toLowerCase()}, {fmt(p.down)} {downLabel.toLowerCase()}">
        <div class="up" style:height="{half}px"><i style:height="{(p.up / max) * 100}%" style:background={upColour}></i></div>
        <div class="down" style:height="{half}px"><i style:height="{(p.down / max) * 100}%" style:background={downColour}></i></div>
        <span class="lab">{label(p.label)}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .key { display: flex; gap: 16px; font-size: 13px; color: var(--t-muted); margin-bottom: 8px; }
  .key span { display: flex; align-items: center; gap: 6px; }
  .key i { width: 10px; height: 10px; border-radius: 3px; }
  .bars { position: relative; display: grid; grid-template-columns: repeat(var(--n), 1fr); gap: 6px; }
  .zero { position: absolute; left: 0; right: 0; height: 1px; background: var(--line); }
  .col { display: flex; flex-direction: column; align-items: center; min-width: 0; }
  .up, .down { width: 100%; display: flex; justify-content: center; }
  .up { align-items: flex-end; }
  .down { align-items: flex-start; }
  .up i, .down i { display: block; width: 100%; max-width: 40px; }
  .up i { border-radius: 4px 4px 0 0; }
  .down i { border-radius: 0 0 4px 4px; }
  .col:hover i { filter: brightness(1.15); }
  .lab { font-size: 12px; color: var(--t-muted); margin-top: 6px; white-space: nowrap; }
</style>
