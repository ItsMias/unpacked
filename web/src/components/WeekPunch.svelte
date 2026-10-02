<script>
  // A punch card of the week: one row per weekday, a dot per hour that's bigger and brighter the
  // more happened then (its area follows the amount). Each day's total runs down the right and
  // each hour's along the bottom; hovering a dot lights up its day and hour.
  //   grid: 7 rows (Monday first) of 24 values, local time
  import { fmt } from "../lib/stats.js";

  let { grid, format = fmt, unit = "", label = "" } = $props();

  const DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
  const ROW = 30, HOURS = 34;
  let width = $state(600);
  let hover = $state(null); // [day, hour]

  // Narrow cards drop the day totals on the right.
  const M = $derived({ top: 4, right: width < 560 ? 8 : 92, bottom: HOURS + 22, left: 38 });
  const cell = $derived((width - M.left - M.right) / 24);
  const height = $derived(M.top + 7 * ROW + M.bottom);
  const cx = (h) => M.left + (h + 0.5) * cell;
  const cy = (d) => M.top + (d + 0.5) * ROW;

  const max = $derived(Math.max(1e-9, ...grid.flat()));
  const days = $derived(grid.map((row) => row.reduce((a, b) => a + b, 0)));
  const hours = $derived(Array.from({ length: 24 }, (_, h) => grid.reduce((s, row) => s + row[h], 0)));
  const maxDay = $derived(Math.max(1e-9, ...days));
  const maxHour = $derived(Math.max(1e-9, ...hours));
  const radius = (v) => Math.max(2, Math.sqrt(v / max) * (Math.min(cell, ROW) / 2 - 1));
  const shade = (v) => `var(--h${Math.min(5, 2 + Math.floor((v / max) * 3.999))})`;
  const hh = (h) => `${String(h % 24).padStart(2, "0")}:00`;
  const tip = (v) => `${format(v)}${unit ? ` ${unit}` : ""}`;
</script>

<div class="punch" bind:clientWidth={width}>
  <svg {width} {height} role="img" aria-label={label} onpointerleave={() => (hover = null)}>
    {#if hover}
      <rect class="band" x={M.left} y={cy(hover[0]) - ROW / 2} width={24 * cell} height={ROW} rx="6" />
      <rect class="band" x={cx(hover[1]) - cell / 2} y={M.top} width={cell} height={7 * ROW + HOURS + 4} rx="6" />
    {/if}

    {#each grid as row, d}
      <text class="day" class:on={hover?.[0] === d} x={M.left - 10} y={cy(d)} dy="0.35em">{DAYS[d].slice(0, 3)}</text>
      {#each row as v, h}
        {#if v > 0}
          <circle cx={cx(h)} cy={cy(d)} r={radius(v)} style:fill={shade(v)} />
        {:else}
          <circle class="none" cx={cx(h)} cy={cy(d)} r="1.5" />
        {/if}
        <rect
          class="hit" x={cx(h) - cell / 2} y={cy(d) - ROW / 2} width={cell} height={ROW}
          data-tip={tip(v)} data-sub="{DAYS[d]}s, {hh(h)}–{hh(h + 1)}"
          onpointerenter={() => (hover = [d, h])}
        />
      {/each}
      {#if M.right > 40}
        <rect class="bar" class:on={hover?.[0] === d} x={width - M.right + 12} y={cy(d) - 4} width={Math.max(2, (days[d] / maxDay) * 34)} height="8" rx="4" />
        <text class="total" x={width - M.right + 52} y={cy(d)} dy="0.35em">{format(days[d])}{unit === "hours" ? " h" : ""}</text>
      {/if}
    {/each}

    {#each hours as t, h}
      {@const bh = Math.max(2, (t / maxHour) * (HOURS - 6))}
      <rect
        class="bar" class:on={hover?.[1] === h}
        x={cx(h) - Math.min(cell * 0.32, 7)} y={M.top + 7 * ROW + 4 + (HOURS - 6 - bh)} width={Math.min(cell * 0.64, 14)} height={bh} rx="2"
        data-tip={tip(t)} data-sub="{hh(h)}–{hh(h + 1)}, all week"
      />
    {/each}
    {#each [0, 3, 6, 9, 12, 15, 18, 21] as h}
      <text class="hour" class:on={hover?.[1] === h} x={cx(h)} y={height - 6}>{cell < 16 ? hh(h).slice(0, 2) : hh(h)}</text>
    {/each}
  </svg>
</div>

<style>
  .punch { width: 100%; }
  svg { display: block; overflow: visible; }
  circle { transition: r .15s; }
  .none { fill: var(--line); }
  .hit { fill: transparent; cursor: default; }
  .band { fill: var(--hover); }
  .day, .total, .hour { font: 600 12px var(--ui); fill: var(--t-muted); }
  .day { text-anchor: end; }
  .day.on, .hour.on { fill: var(--t-head); }
  .total { fill: var(--t-body); font-variant-numeric: tabular-nums; }
  .hour { font-size: 11px; font-weight: 500; fill: var(--t-faint); text-anchor: middle; }
  .bar { fill: var(--acc); opacity: .45; }
  .bar.on { opacity: 1; }
</style>
