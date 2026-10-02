<script>
  // A 24-hour clock with one bar per hour: how much happens at each time of day.
  //   hours: 24 numbers (local time), format: label for a value
  let { hours, format = (n) => String(n), label = "", unit = "" } = $props();

  const S = 300, C = S / 2, R0 = 46, R1 = 136;
  const max = $derived(Math.max(...hours, 1e-9));
  const peak = $derived(hours.indexOf(Math.max(...hours)));
  const hh = (h) => `${String(h % 24).padStart(2, "0")}:00`;

  function wedge(h, v) {
    const r = R0 + (v / max) * (R1 - R0);
    const a0 = ((h - 6) / 24) * 2 * Math.PI + 0.025, a1 = ((h - 5) / 24) * 2 * Math.PI - 0.025;
    const p = (rad, a) => `${C + rad * Math.cos(a)},${C + rad * Math.sin(a)}`;
    return `M${p(R0, a0)} L${p(r, a0)} A${r},${r} 0 0 1 ${p(r, a1)} L${p(R0, a1)} A${R0},${R0} 0 0 0 ${p(R0, a0)} Z`;
  }
</script>

<svg viewBox="0 0 {S} {S}" role="img" aria-label={label}>
  {#each [0.5, 1] as f}
    <circle cx={C} cy={C} r={R0 + f * (R1 - R0)} class="ring" />
  {/each}
  {#each hours as v, h}
    <path d={wedge(h, v)} class:peak={h === peak} data-tip="{format(v)}{unit}" data-sub="{hh(h)}–{hh(h + 1)}" />
  {/each}
  {#each [0, 6, 12, 18] as h}
    {@const a = ((h - 6) / 24) * 2 * Math.PI}
    <text x={C + (R1 + 10) * Math.cos(a)} y={C + (R1 + 10) * Math.sin(a)} dy="0.35em">{String(h).padStart(2, "0")}</text>
  {/each}
  <text class="c1" x={C} y={C - 4}>{hh(peak)}</text>
  <text class="c2" x={C} y={C + 14}>busiest</text>
</svg>

<style>
  svg { display: block; width: 100%; max-width: 320px; margin: 0 auto; overflow: visible; }
  .ring { fill: none; stroke: var(--line); stroke-dasharray: 2 4; }
  path { fill: var(--h3); }
  path.peak { fill: var(--acc-text); }
  path:hover { filter: brightness(1.25); }
  text { fill: var(--t-faint); font: 600 11px var(--ui); text-anchor: middle; }
  .c1 { fill: var(--t-head); font: 900 18px var(--display); }
  .c2 { fill: var(--t-muted); font: 600 11px var(--ui); }
</style>
