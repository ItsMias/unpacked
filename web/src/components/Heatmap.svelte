<script>
  // One calendar year of daily message counts, Monday first, GitHub style.
  import { fmt, level, longDate } from "../lib/stats.js";

  /** `from` / `until`: day keys outside the account's life, drawn as empty slots. */
  let { daily, breaks, year, from = "", until = "9999" } = $props();

  const DAY = 86_400_000;
  const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  const LABELS = ["Mon", "", "Wed", "", "Fri", "", ""];

  const grid = $derived.by(() => {
    const first = Date.UTC(year, 0, 1);
    const start = first - ((new Date(first).getUTCDay() + 6) % 7) * DAY;
    const col = (t) => Math.floor((t - start) / DAY / 7) + 2;
    const cells = [];
    for (let t = first; new Date(t).getUTCFullYear() === year; t += DAY) {
      const key = new Date(t).toISOString().slice(0, 10);
      const n = daily.get(key) ?? 0;
      cells.push({ key, n, l: level(n, breaks), out: key < from || key > until, col: col(t), row: ((new Date(t).getUTCDay() + 6) % 7) + 2 });
    }
    const months = MONTHS.map((label, m) => ({ label, col: col(Date.UTC(year, m, 1)) }));
    return { cells, months, weeks: col(Date.UTC(year, 11, 31)) - 1 };
  });
</script>

<div class="cal">
  <div class="grid" style:--weeks={grid.weeks} role="img" aria-label="Messages per day in {year}">
    {#each LABELS as label, i}
      <span class="day" style:grid-row={i + 2}>{label}</span>
    {/each}
    {#each grid.months as m}
      <span class="month" style:grid-column="{m.col} / span 3">{m.label}</span>
    {/each}
    {#each grid.cells as c (c.key)}
      {#if c.out}
        <i class="cell out" style:grid-column={c.col} style:grid-row={c.row}></i>
      {:else}
        <i
          class="cell"
          data-l={c.l}
          style:grid-column={c.col}
          style:grid-row={c.row}
          data-tip="{c.n ? fmt(c.n) : 'No'} message{c.n === 1 ? '' : 's'}"
          data-sub={longDate(c.key)}
        ></i>
      {/if}
    {/each}
  </div>
</div>

<style>
  .cal { overflow-x: auto; padding-bottom: 4px; }
  /* Fluid: the year fills its card; below ~620px it scrolls sideways inside the card. */
  .grid {
    display: grid;
    grid-template-columns: 28px repeat(var(--weeks), minmax(0, 1fr));
    grid-template-rows: 16px;
    gap: 3px;
    min-width: 620px;
  }
  .month { grid-row: 1; font-size: 11px; color: var(--t-muted); white-space: nowrap; }
  .day { grid-column: 1; font-size: 10px; color: var(--t-faint); line-height: 1; align-self: center; }
  .cell { aspect-ratio: 1; }
  .cell:not(.out):hover { outline: 2px solid var(--t-head); outline-offset: 1px; }
  .out { background: none; box-shadow: inset 0 0 0 1px var(--sunk); }
</style>
