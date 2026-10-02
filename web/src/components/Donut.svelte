<script>
  // Part-to-whole ring with a legend. segments: [{ label, value, colour }]
  import { fmt } from "../lib/stats.js";

  let { segments, centre = "", centreSub = "", label = "" } = $props();

  const R = 70, W = 22, C = 2 * Math.PI * R;
  const total = $derived(segments.reduce((s, x) => s + x.value, 0));
  const arcs = $derived.by(() => {
    let at = 0;
    return segments.filter((s) => s.value > 0).map((s) => {
      const len = (s.value / Math.max(total, 1)) * C;
      // A 2px gap between segments, unless there's only one.
      const arc = { ...s, dash: `${Math.max(len - (segments.length > 1 ? 2 : 0), 0.5)} ${C}`, offset: -at };
      at += len;
      return arc;
    });
  });
  const pct = (v) => `${((v / Math.max(total, 1)) * 100).toFixed(v / Math.max(total, 1) < 0.1 ? 1 : 0)}%`;
</script>

<div class="donut">
  <svg viewBox="0 0 180 180" width="180" height="180" role="img" aria-label={label}>
    <circle cx="90" cy="90" r={R} class="track" stroke-width={W} />
    {#each arcs as a (a.label)}
      <circle
        cx="90" cy="90" r={R} stroke-width={W}
        style:stroke={a.colour}
        stroke-dasharray={a.dash}
        stroke-dashoffset={a.offset}
        transform="rotate(-90 90 90)"
        data-tip="{fmt(a.value)} {a.label.toLowerCase()}"
        data-sub={pct(a.value)}
      />
    {/each}
    <text x="90" y="88" class="c1">{centre}</text>
    <text x="90" y="108" class="c2">{centreSub}</text>
  </svg>
  <ul>
    {#each segments as s (s.label)}
      <li><i style:background={s.colour}></i><span>{s.label}</span><b class="num">{fmt(s.value)}</b><small class="num">{pct(s.value)}</small></li>
    {/each}
  </ul>
</div>

<style>
  .donut { display: flex; align-items: center; gap: 24px; flex-wrap: wrap; }
  svg { flex: none; }
  circle { fill: none; }
  .track { stroke: var(--sunk); }
  circle[data-tip]:hover { filter: brightness(1.15); }
  .c1 { fill: var(--t-head); font: 900 22px var(--display); text-anchor: middle; }
  .c2 { fill: var(--t-muted); font: 600 12px var(--ui); text-anchor: middle; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; flex: 1; min-width: 180px; }
  li { display: grid; grid-template-columns: 10px 1fr auto 48px; align-items: center; gap: 10px; font-size: 14px; }
  li i { width: 10px; height: 10px; border-radius: 3px; }
  li b { color: var(--t-head); }
  li small { color: var(--t-muted); text-align: right; }
</style>
