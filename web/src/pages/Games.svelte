<script>
  import AreaChart from "../components/AreaChart.svelte";
  import RadialHours from "../components/RadialHours.svelte";
  import { appInfo } from "../lib/discord.js";
  import { fmt, gameStats, monthYear } from "../lib/stats.js";

  let { result, view, range, games } = $props();

  const facts = $derived(result.facts);
  const s = $derived(gameStats(view, facts, games, range));
  const hours = (n) => (n >= 100 ? fmt(n) : n >= 10 ? n.toFixed(0) : n.toFixed(1).replace(/\.0$/, ""));
  const poker = $derived(facts.poker);
  const stats = $derived(facts.account?.app_stats ?? []);

  // Names, icons and Steam covers for everything shown, looked up once each. Covers that fail to
  // load fall back to the blurred icon.
  let apps = $state({});
  let broken = $state({});
  const asked = new Set();
  function want(id) {
    if (!id || asked.has(id)) return;
    asked.add(id);
    appInfo(id).then((a) => a && (apps = { ...apps, [id]: a }));
  }
  $effect(() => {
    for (const g of s?.list.slice(0, 12) ?? []) want(g.id);
    for (const a of stats.slice(0, 12)) want(a.id);
    for (const [id] of facts.activities.slice(0, 8)) want(id);
  });
</script>

<div class="page">
  {#if !s}
    <section class="card"><h2>Games</h2><p class="muted">Your package doesn't show any games.</p></section>
  {:else}
    <section class="strip">
      <dl>
        {#if s.analytics}<div><dt>Hours played</dt><dd>{fmt(s.total)}</dd></div>{/if}
        <div><dt>Games</dt><dd>{fmt(s.distinct)}</dd></div>
        <div><dt>Most played</dt><dd class="name">{s.list[0]?.name ?? "–"}</dd></div>
        {#if facts.activities.length}<div><dt>Activities joined</dt><dd>{fmt(facts.activities.reduce((a, x) => a + x[1], 0))}</dd></div>{/if}
        {#if poker?.games_won}<div><dt>Poker wins</dt><dd>{fmt(poker.games_won)}</dd></div>{/if}
      </dl>
    </section>
    {#if !s.analytics}
      <p class="note">Without the analytics folder, only games you played while in a voice channel show up, and without playtime.</p>
    {/if}

    <section class="covers">
      {#each s.list.slice(0, 12) as g, i (g.name)}
        {@const a = apps[g.id]}
        <article class:wide={i === 0}>
          <div class="art">
            {#if a?.cover && !broken[g.id]}
              <img class="cover" src={a.cover} alt="" loading="lazy" onerror={() => (broken = { ...broken, [g.id]: true })} />
            {:else if a?.icon}
              <img class="blur" src={a.icon} alt="" aria-hidden="true" />
            {/if}
            {#if a?.icon}<img class="icon" src={a.icon} alt="" loading="lazy" />{:else}<span class="ph">{[...g.name][0]}</span>{/if}
          </div>
          <div class="info">
            <strong>{g.name}</strong>
            <small>
              {#if s.analytics}{hours(g.rangeHours)} h · {/if}{fmt(g.sessions)} session{g.sessions === 1 ? "" : "s"}
            </small>
            {#if g.first_ms}<small>{monthYear(g.first_ms)} – {monthYear(g.last_ms)}</small>{/if}
          </div>
        </article>
      {/each}
    </section>

    <div class="grid12">
      {#if s.analytics && s.points.length > 1}
        <section class="card s12">
          <div class="chead">
            <h2>Playtime per month</h2>
            <div class="key">{#each s.series as x (x.key)}<span><i style:background={x.colour}></i>{x.label}</span>{/each}</div>
          </div>
          <AreaChart points={s.points} series={s.series} format={hours} label="Hours played per month, by game" />
        </section>
      {/if}

      {#if s.clock}
        <section class="card s5">
          <h2>When you play</h2>
          <RadialHours hours={s.clock.clock} format={hours} unit=" hours" label="Hours played by hour of day" />
        </section>
      {/if}

      {#if stats.length}
        <section class="card {s.clock ? 's7' : 's12'}">
          <h2>Discord's own count <small>(kept on your account)</small></h2>
          <ol class="rank">
            {#each stats.slice(0, 10) as x (x.id)}
              {@const a = apps[x.id]}
              <li>
                {#if a?.icon}<img src={a.icon} alt="" width="32" height="32" />{:else}<span class="ph sm"></span>{/if}
                <div>
                  <strong>{a?.name ?? "Game"}</strong>
                  <span class="meter"><i style:width="{(x.seconds / stats[0].seconds) * 100}%"></i></span>
                </div>
                <span class="n num">{hours(x.seconds / 3600)} h</span>
              </li>
            {/each}
          </ol>
        </section>
      {/if}

      {#if facts.activities.length}
        <section class="card s7">
          <h2>Activities</h2>
          <ol class="rank">
            {#each facts.activities.slice(0, 8) as [id, n, first, last] (id)}
              {@const a = apps[id]}
              <li>
                {#if a?.icon}<img src={a.icon} alt="" width="32" height="32" />{:else}<span class="ph sm"></span>{/if}
                <div>
                  <strong>{a?.name ?? "Activity"}</strong>
                  <small>{monthYear(first)} – {monthYear(last)}</small>
                </div>
                <span class="n num">{fmt(n)}×</span>
              </li>
            {/each}
          </ol>
        </section>
      {/if}

      {#if poker}
        <section class="card s5">
          <h2>Poker Night</h2>
          <dl class="poker">
            <div><dt>Games</dt><dd>{fmt(poker.games_played ?? 0)}</dd></div>
            <div><dt>Won</dt><dd>{fmt(poker.games_won ?? 0)}<small>{poker.games_played ? ` ${Math.round((poker.games_won / poker.games_played) * 100)}%` : ""}</small></dd></div>
            <div><dt>All-ins</dt><dd>{fmt(poker.all_ins ?? 0)}</dd></div>
            <div><dt>Biggest pot</dt><dd>{fmt(poker.biggest_pot ?? 0)}</dd></div>
            <div><dt>Most chips</dt><dd>{fmt(poker.most_chips ?? 0)}</dd></div>
            <div><dt>Level</dt><dd>{fmt(poker.level ?? 0)}</dd></div>
          </dl>
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .name { font-size: 20px !important; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .note { margin: 12px 0 0; padding: 12px 14px; border-radius: 8px; background: var(--acc-soft); font-size: 14px; }

  .covers { display: grid; grid-template-columns: repeat(6, 1fr); gap: 12px; margin-top: 16px; }
  .covers article { background: var(--card); border-radius: var(--radius); overflow: hidden; min-width: 0; }
  .covers article.wide { grid-column: span 2; grid-row: span 2; }
  .art { position: relative; aspect-ratio: 2 / 3; background: linear-gradient(160deg, var(--raised), var(--sunk)); display: grid; place-items: center; }
  .wide .art { aspect-ratio: auto; height: calc(100% - 72px); min-height: 240px; }
  .art { overflow: hidden; }
  .cover { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
  /* No cover art: the icon, blown up and blurred, fills the space behind the sharp one. */
  .blur { position: absolute; inset: -20%; width: 140%; height: 140%; object-fit: cover; filter: blur(28px) saturate(1.3); opacity: .55; }
  .wide .icon { width: 88px; height: 88px; border-radius: 20px; }
  .icon { width: 56px; height: 56px; border-radius: 14px; position: relative; }
  .cover + .icon { position: absolute; left: 10px; bottom: 10px; width: 36px; height: 36px; border-radius: 10px; box-shadow: 0 2px 8px rgba(0, 0, 0, .5); }
  .ph { width: 56px; height: 56px; border-radius: 14px; background: var(--raised); display: grid; place-items: center; font: 900 24px var(--display); color: var(--t-muted); }
  .ph.sm { width: 32px; height: 32px; border-radius: 8px; }
  .info { padding: 10px 12px 12px; }
  .info strong { display: block; color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .info small { display: block; color: var(--t-muted); font-size: 12px; }

  .key { display: flex; gap: 14px; flex-wrap: wrap; font-size: 13px; color: var(--t-muted); }
  .key span { display: flex; align-items: center; gap: 6px; }
  .key i { width: 10px; height: 10px; border-radius: 3px; }

  .rank { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .rank li { display: grid; grid-template-columns: 32px 1fr auto; align-items: center; gap: 12px; }
  .rank img { border-radius: 8px; display: block; }
  .rank div { min-width: 0; }
  .rank strong { display: block; color: var(--t-head); font-weight: 600; margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rank small { color: var(--t-muted); font-size: 12px; }
  .n { color: var(--t-muted); font-size: 13px; }

  .poker { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin: 0; }
  .poker dt { font-size: 13px; color: var(--t-muted); font-weight: 600; }
  .poker dd { margin: 2px 0 0; font: 900 22px var(--display); color: var(--t-head); }
  .poker small { font: 600 13px var(--ui); color: var(--acc-text); }

  @media (max-width: 900px) {
    .covers { grid-template-columns: repeat(3, 1fr); }
    .covers article.wide { grid-column: span 3; grid-row: auto; }
    .wide .art { height: 200px; min-height: 0; }
  }
</style>
