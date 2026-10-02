<script>
  // A ranked list's title and column headings, sorting like a file manager's: click a heading
  // to sort by it, click it again to flip the order; the arrow shows the column and direction.
  // It lines up with the rows through the `--cols` grid the list's card sets: the title takes the
  // first three columns (rank, picture, name), then one heading per column after that.
  //   columns: [{ key, label, asc? }]  asc: the column starts lowest/oldest first
  import Icon from "./Icon.svelte";

  let { title, columns, sort = $bindable(), desc = $bindable(), onsort = () => {} } = $props();

  function pick(c) {
    if (sort === c.key) desc = !desc;
    else { sort = c.key; desc = !c.asc; }
    onsort();
  }
  // Which way the arrow points: the current order on the sorted column, the first click's elsewhere.
  const down = (c) => (sort === c.key ? desc : !c.asc);
</script>

<div class="heads">
  <h2>{title}</h2>
  {#each columns as c (c.key)}
    <button class="head" class:on={sort === c.key} aria-pressed={sort === c.key} onclick={() => pick(c)}>
      {c.label}
      <span class="arrow" class:up={!down(c)}><Icon name="arrow" size={13} stroke={2.4} /></span>
      {#if sort === c.key}<span class="sr-only">, {desc ? "highest first" : "lowest first"}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .heads {
    display: grid; grid-template-columns: var(--cols); align-items: end; gap: 14px;
    padding: 0 8px 10px; margin-bottom: 6px; border-bottom: 1px solid var(--line);
  }
  h2 { grid-column: 1 / 4; margin: 0; font-size: 18px; font-weight: 700; color: var(--t-head); }
  .head {
    justify-self: start; display: inline-flex; align-items: center; gap: 4px; padding: 2px 0; border: 0; background: none;
    font: 600 13px var(--ui); color: var(--t-muted); white-space: nowrap; cursor: pointer;
  }
  .head:hover { color: var(--t-body); }
  .head.on { color: var(--t-head); }
  .arrow { display: grid; color: var(--acc-text); opacity: 0; transition: opacity .15s, rotate .2s; }
  .arrow.up { rotate: 180deg; }
  .head:hover .arrow { opacity: .55; }
  .head.on .arrow { opacity: 1; }

  /* On phones the rows only show the sorted column, so the headings become a row of choices. */
  @media (max-width: 760px) {
    .heads { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 14px; }
    h2 { flex: 1 0 100%; margin-bottom: 4px; }
  }
</style>
