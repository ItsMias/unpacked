<script>
  // Stacked area chart over time, with a crosshair tooltip. One series draws as a filled line.
  //   points: [{ t: UTC ms of the bucket start, [series.key]: number }]
  //   series: [{ key, label, colour (CSS colour, or "hatch") }], bottom layer first
  let { points, series, weekly = false, height = 220, format = (n) => String(n), label = "" } = $props();

  const M = { top: 10, right: 10, bottom: 24, left: 46 };
  const uid = Math.random().toString(36).slice(2, 8);
  let width = $state(600);
  let hover = $state(null);

  const innerW = $derived(Math.max(width - M.left - M.right, 10));
  const innerH = $derived(height - M.top - M.bottom);
  const step = $derived(points.length > 1 ? innerW / (points.length - 1) : innerW);
  const x = (i) => M.left + i * step;

  const totals = $derived(points.map((p) => series.reduce((s, ser) => s + (p[ser.key] ?? 0), 0)));
  // A round top value with ~4 gridlines.
  const scale = $derived.by(() => {
    const max = Math.max(...totals, 1);
    const raw = max / 4;
    const mag = 10 ** Math.floor(Math.log10(raw));
    const s = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((v) => v >= raw);
    return { step: s, max: s * Math.ceil(max / s) };
  });
  const y = (v) => M.top + innerH - (v / scale.max) * innerH;
  const ticks = $derived(Array.from({ length: Math.round(scale.max / scale.step) + 1 }, (_, i) => i * scale.step));

  const layers = $derived.by(() => {
    const base = points.map(() => 0);
    return series.map((ser) => {
      const lower = [...base];
      points.forEach((p, i) => (base[i] += p[ser.key] ?? 0));
      const upper = [...base];
      const top = upper.map((v, i) => `${x(i)},${y(v)}`).join(" L");
      const bottom = lower.map((v, i) => `${x(i)},${y(v)}`).reverse().join(" L");
      return { ...ser, area: `M${top} L${bottom} Z`, line: `M${top}` };
    });
  });

  const MONTH = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  const date = (t) => new Date(t);
  const xLabels = $derived.by(() => {
    const out = [];
    const span = points.length;
    points.forEach((p, i) => {
      const d = date(p.t);
      if (weekly) {
        const prev = i ? date(points[i - 1].t).getUTCMonth() : -1;
        if (d.getUTCMonth() !== prev) out.push({ i, text: MONTH[d.getUTCMonth()] });
      } else if (span > 30 ? d.getUTCMonth() === 0 : d.getUTCMonth() % 3 === 0) {
        out.push({ i, text: d.getUTCMonth() === 0 ? String(d.getUTCFullYear()) : MONTH[d.getUTCMonth()] });
      }
    });
    // Skip labels that would run into the one before (long histories on narrow screens).
    let last = -Infinity;
    return out.filter((l) => {
      if (x(l.i) - last < 44) return false;
      last = x(l.i);
      return true;
    });
  });
  const when = (t) => {
    const d = date(t);
    return weekly ? `Week of ${d.getUTCDate()} ${MONTH[d.getUTCMonth()]} ${d.getUTCFullYear()}` : `${MONTH[d.getUTCMonth()]} ${d.getUTCFullYear()}`;
  };

  function move(e) {
    const r = e.currentTarget.getBoundingClientRect();
    const i = Math.round((e.clientX - r.left - M.left) / step);
    hover = i >= 0 && i < points.length ? i : null;
  }
</script>

<div class="chart" bind:clientWidth={width} style:height="{height}px">
  <svg {width} {height} role="img" aria-label={label} onpointermove={move} onpointerleave={() => (hover = null)}>
    <defs>
      <pattern id="hatch-{uid}" width="5" height="5" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
        <rect width="5" height="5" fill="var(--sunk)" />
        <line x1="0" y1="0" x2="0" y2="5" stroke="var(--t-faint)" stroke-width="2" />
      </pattern>
    </defs>
    {#each ticks as t}
      <line class="grid" x1={M.left} x2={width - M.right} y1={y(t)} y2={y(t)} />
      <text class="ylab" x={M.left - 8} y={y(t)} dy="0.32em">{format(t)}</text>
    {/each}
    {#each xLabels as l}
      <text class="xlab" x={x(l.i)} y={height - 6}>{l.text}</text>
    {/each}
    {#each layers as l, i (l.key)}
      {@const fill = l.colour === "hatch" ? `url(#hatch-${uid})` : l.colour}
      <path d={l.area} style:fill={fill} style:opacity={series.length === 1 ? 0.3 : 1} />
      {#if series.length === 1}
        <path class="line" d={l.line} style:stroke={l.colour} />
      {:else if i < layers.length - 1}
        <path class="gap" d={l.line} />
      {/if}
    {/each}
    {#if hover !== null}
      <line class="cross" x1={x(hover)} x2={x(hover)} y1={M.top} y2={M.top + innerH} />
      <circle cx={x(hover)} cy={y(totals[hover])} r="4" />
    {/if}
  </svg>
  {#if hover !== null}
    {@const p = points[hover]}
    <div class="tipbox" class:left={x(hover) > width * 0.6} style:left="{x(hover)}px">
      <strong>{when(p.t)}</strong>
      {#each [...series].reverse() as s (s.key)}
        <div><i class="sw" class:hatch={s.colour === "hatch"} style:background={s.colour === "hatch" ? null : s.colour}></i>{s.label}<b class="num">{format(p[s.key] ?? 0)}</b></div>
      {/each}
      {#if series.length > 1}<div class="total">Total<b class="num">{format(totals[hover])}</b></div>{/if}
    </div>
  {/if}
</div>

<style>
  .chart { position: relative; width: 100%; }
  svg { display: block; overflow: visible; touch-action: pan-y; }
  .grid { stroke: var(--line); stroke-width: 1; }
  .ylab, .xlab { fill: var(--t-faint); font-size: 11px; font-family: var(--ui); }
  .ylab { text-anchor: end; }
  .xlab { text-anchor: middle; }
  .line { fill: none; stroke-width: 2; stroke-linejoin: round; }
  .gap { fill: none; stroke: var(--card); stroke-width: 2; stroke-linejoin: round; }
  .cross { stroke: var(--t-muted); stroke-width: 1; stroke-dasharray: 3 3; }
  circle { fill: var(--t-head); stroke: var(--card); stroke-width: 2; }

  .tipbox {
    position: absolute; top: 0; transform: translateX(12px); pointer-events: none; z-index: 5;
    background: #111214; border-radius: 6px; padding: 8px 10px; min-width: 170px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, .24); font-size: 13px; color: var(--t-body);
  }
  .tipbox.left { transform: translateX(calc(-100% - 12px)); }
  .tipbox strong { display: block; color: var(--t-head); margin-bottom: 4px; }
  .tipbox div { display: flex; align-items: center; gap: 6px; }
  .tipbox b { margin-left: auto; padding-left: 12px; color: var(--t-head); }
  .tipbox .total { border-top: 1px solid var(--line); margin-top: 4px; padding-top: 4px; color: var(--t-muted); }
  .sw { width: 9px; height: 9px; border-radius: 2px; flex: none; }
  .sw.hatch { background: repeating-linear-gradient(135deg, var(--t-faint) 0 2px, transparent 2px 4px); box-shadow: inset 0 0 0 1px var(--t-faint); }
</style>
