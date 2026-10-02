<script>
  import Icon from "../components/Icon.svelte";
  import { statGroups } from "../lib/eventStats.js";
  import { fmt } from "../lib/stats.js";

  let { result, range, games } = $props();

  const groups = $derived(statGroups(result, games));
</script>

<div class="page">
  {#if range !== "all"}<p class="note">These are all-time counts: the logs only keep a total per kind of event.</p>{/if}
  {#if games?.state === "reading"}<p class="note">Still reading the analytics log; more counts appear when it's done.</p>{/if}

  {#each groups as g (g.title)}
    <section class="card">
      <h2><span class="ic"><Icon name={g.icon} size={18} /></span>{g.title}</h2>
      <div class="tiles">
        {#each g.items as x (x.label)}
          <div class="tile"><b class="num">{fmt(x.n)}</b><span>{x.label}</span></div>
        {/each}
      </div>
    </section>
  {:else}
    <section class="card"><h2>Statistics</h2><p class="muted">Your package has no activity logs to count.</p></section>
  {/each}
</div>

<style>
  .page { display: grid; gap: 16px; }
  .note { margin: 0; padding: 10px 14px; border-radius: 8px; background: var(--card); color: var(--t-muted); font-size: 14px; }
  h2 { display: flex; align-items: center; gap: 10px; }
  .ic { width: 32px; height: 32px; border-radius: 10px; display: grid; place-items: center; background: var(--acc-soft); color: var(--acc-text); }
  .tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 10px; }
  .tile { display: grid; gap: 4px; padding: 14px 16px; border-radius: 10px; background: var(--sunk); min-width: 0; }
  .tile b { font: 900 24px/1.1 var(--display); color: var(--t-head); }
  .tile span { font-size: 13px; font-weight: 600; color: var(--t-muted); }
  @media (max-width: 620px) {
    .tiles { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .tile b { font-size: 20px; }
  }
</style>
