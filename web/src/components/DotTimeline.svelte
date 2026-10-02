<script>
  // Dots on a time axis, higher for bigger values: when the notable things happened.
  //   points: [{ ms, value, tip, sub, key }]
  let { points, from, to, height = 140, format = (n) => String(n), onpick = null, active = null } = $props();

  const M = { top: 12, right: 12, bottom: 24, left: 52 };
  let width = $state(600);
  // Dots don't need a zero baseline: the scale starts a bit under the smallest value, so close
  // values still spread out.
  const max = $derived(Math.max(1, ...points.map((p) => p.value)));
  const min = $derived.by(() => {
    const lo = Math.min(...points.map((p) => p.value));
    const pad = Math.max((max - lo) * 0.25, max * 0.01);
    return Math.max(0, Math.floor((lo - pad) / 10) * 10);
  });
  const span = $derived(Math.max(1, to - from));
  const x = (ms) => M.left + ((ms - from) / span) * (width - M.left - M.right);
  const y = (v) => M.top + (1 - (v - min) / Math.max(1, max - min)) * (height - M.top - M.bottom);
  const years = $derived.by(() => {
    const out = [];
    for (let yv = new Date(from).getUTCFullYear() + 1; Date.UTC(yv, 0, 1) <= to; yv++) out.push(yv);
    return out;
  });
  const ticks = $derived([min, Math.round((min + max) / 2), max]);
</script>

<div class="dots" bind:clientWidth={width}>
  <svg {width} {height} role="img" aria-label="When they were sent">
    {#each ticks as t}
      <line class="grid" x1={M.left} x2={width - M.right} y1={y(t)} y2={y(t)} />
      <text class="ylab" x={M.left - 8} y={y(t)} dy="0.32em">{format(t)}</text>
    {/each}
    {#each years as yv}
      <line class="tick" x1={x(Date.UTC(yv, 0, 1))} x2={x(Date.UTC(yv, 0, 1))} y1={M.top} y2={height - M.bottom} />
      <text class="xlab" x={x(Date.UTC(yv, 0, 1))} y={height - 6}>{yv}</text>
    {/each}
    {#each points as p (p.key)}
      <circle
        cx={x(p.ms)} cy={y(p.value)} r={active === p.key ? 8 : 6}
        class:on={active === p.key}
        data-tip={p.tip} data-sub={p.sub}
        role="button"
        tabindex="0"
        onclick={() => onpick?.(p.key)}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onpick?.(p.key)}
      />
    {/each}
  </svg>
</div>

<style>
  .dots { width: 100%; }
  svg { display: block; overflow: visible; }
  .grid { stroke: var(--line); }
  .tick { stroke: var(--sunk); }
  .ylab, .xlab { fill: var(--t-faint); font: 11px var(--ui); }
  .ylab { text-anchor: end; }
  .xlab { text-anchor: middle; }
  circle { fill: var(--acc); stroke: var(--card); stroke-width: 2; cursor: pointer; }
  circle:hover, circle.on { fill: var(--acc-text); }
  circle:focus-visible { outline: none; stroke: var(--t-head); }
</style>
