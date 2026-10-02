<script>
  import AreaChart from "../components/AreaChart.svelte";
  import DivergingBars from "../components/DivergingBars.svelte";
  import Donut from "../components/Donut.svelte";
  import ServerIcon from "../components/ServerIcon.svelte";
  import SortHeads from "../components/SortHeads.svelte";
  import TagChip from "../components/TagChip.svelte";
  import { serverInfo } from "../lib/discord.js";
  import { allTags, fmt, monthYear, serverStats, shortDate } from "../lib/stats.js";

  let { result, view, range, games } = $props();

  const facts = $derived(result.facts);
  const s = $derived(serverStats(view, facts, range, result.sources));

  // Sortable columns. Joined starts oldest first, the rest biggest first; servers without a value
  // for the column go last either way.
  const COLUMNS = [
    { key: "messages", label: "Messages", value: (x) => x.messages },
    { key: "voice", label: "Hours in voice", value: (x) => x.voice },
    { key: "mod", label: "Mod actions", value: (x) => x.mod },
    { key: "joined", label: "Joined", value: (x) => x.joined ?? 0, asc: true },
  ];
  let sort = $state("messages");
  let desc = $state(true);
  // The top 15, or every server in a scrolling list.
  let all = $state(false);
  let list = $state();
  const sorted = $derived.by(() => {
    const value = COLUMNS.find((c) => c.key === sort).value;
    return [...s.ranked].sort((a, b) => {
      const x = value(a), y = value(b);
      return !x || !y ? !x - !y : desc ? y - x : x - y;
    });
  });
  const top = $derived(all ? sorted : sorted.slice(0, 15));
  const max = $derived(Math.max(1, ...s.ranked.map((x) => x.messages)));
  const hours = (n) => (n >= 10 ? `${fmt(n)} h` : n >= 0.05 ? `${n.toFixed(1)} h` : "–");

  // Icons: from the package for servers you manage, otherwise looked up through invites, each
  // once its row scrolls into view.
  let icons = $state({});
  const requested = new Set();
  function iconWhenSeen(node, x) {
    const io = new IntersectionObserver(([e]) => {
      if (!e.isIntersecting) return;
      io.disconnect();
      if (requested.has(x.id) || !x.info) return;
      requested.add(x.id);
      serverInfo(x.info, result.icons?.[x.id]).then((i) => (icons = { ...icons, [x.id]: i }));
    }, { rootMargin: "200px" });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  const MOD = { add_role: "Roles given", remove_role: "Roles removed", ban: "Bans", kick: "Kicks", mute: "Mutes", timeout: "Timeouts", change_nickname: "Nicknames changed" };
  const mods = $derived.by(() => {
    const top4 = s.mods.slice(0, 4).map(([k, n], i) => ({ label: MOD[k] ?? k.replace(/_/g, " "), value: n, colour: `var(--cat-${i + 1})` }));
    const rest = s.mods.slice(4).reduce((a, [, n]) => a + n, 0);
    return rest ? [...top4, { label: "Other", value: rest, colour: "var(--t-faint)" }] : top4;
  });
  const names = $derived(new Map(facts.servers.map((x) => [x.id, x.name])));
  const made = $derived(allTags(facts, games).filter((t) => t.kind === "created" && t.tag && (range === "all" || new Date(t.ms).getUTCFullYear() === range)));
</script>

<div class="page">
  <section class="strip">
    <dl>
      <div><dt>Servers now</dt><dd>{fmt(result.sources?.serversNow ?? 0)}</dd></div>
      <div><dt>Joined</dt><dd>{fmt(s.joined)}</dd></div>
      <div><dt>Left</dt><dd>{fmt(s.left)}</dd></div>
      <div><dt>Created</dt><dd>{fmt(s.created)}</dd></div>
      <div><dt>Boosts</dt><dd>{fmt(s.boosts)}</dd></div>
      <div><dt>Bots added</dt><dd>{fmt(s.bots)}</dd></div>
    </dl>
  </section>

  <div class="grid12">
    <section class="card s12 list">
      <SortHeads title="Your servers" columns={COLUMNS} bind:sort bind:desc onsort={() => list?.scrollTo(0, 0)} />
      <ol class="servers" class:scroll-list={all} bind:this={list}>
        {#each top as x, i (x.id)}
          <li use:iconWhenSeen={x}>
            <span class="rk">{i + 1}</span>
            <ServerIcon url={icons[x.id]?.icon} name={icons[x.id]?.name ?? x.name} size={40} />
            <div class="who">
              <strong>{x.info?.name_known === false && icons[x.id]?.name ? icons[x.id].name : x.name}</strong>
              <small>{x.member ? "Member" : "Left or gone"}</small>
            </div>
            <div class="msgs" class:on={sort === "messages"}>
              <b class="num">{fmt(x.messages)}</b>
              <span class="meter"><i style:width="{(x.messages / max) * 100}%"></i></span>
            </div>
            <b class="stat num" class:on={sort === "voice"}>{hours(x.voice)}</b>
            <b class="stat num" class:on={sort === "mod"}>{x.mod ? fmt(x.mod) : "–"}</b>
            <b class="stat" class:on={sort === "joined"}>{x.joined ? monthYear(x.joined) : "–"}</b>
          </li>
        {:else}
          <li class="muted">No server activity in this range.</li>
        {/each}
      </ol>
      {#if sorted.length > 15}
        <button class="more" onclick={() => (all = !all)}>{all ? "Show fewer" : `Show all ${fmt(sorted.length)}`}</button>
      {/if}
    </section>

    {#if s.counts.length > 1}
      <section class="card s12">
        <h2>Servers you were in</h2>
        <AreaChart points={s.counts} series={[{ key: "servers", label: "Servers", colour: "var(--acc)" }]} height={200} format={fmt} label="Servers you were in, per month" />
      </section>
    {/if}

    <section class="card s7">
      <h2>Joins and leaves</h2>
      {#if s.joinYears.length}
        <DivergingBars points={s.joinYears} upLabel="Joined" downLabel="Left" />
      {:else}
        <p class="muted">No joins or leaves in this range.</p>
      {/if}
    </section>

    <section class="card s5">
      <h2>Moderation</h2>
      {#if s.modTotal}
        <Donut segments={mods} centre={fmt(s.modTotal)} centreSub="actions" label="Moderation actions by type" />
      {:else}
        <p class="muted">No moderation actions in this range.</p>
      {/if}
    </section>

    {#if made.length}
      <section class="card s12">
        <h2>Server tags you set up</h2>
        <ol class="tags">
          {#each [...made].reverse() as t (t.ms)}
            <li>
              <TagChip tag={t.tag} badge={t.badge} colours={t.colours} />
              <span>{names.get(t.guild) ?? "a server"}</span>
              <small>{shortDate(t.ms)}</small>
            </li>
          {/each}
        </ol>
      </section>
    {/if}
  </div>
</div>

<style>
  .list { --cols: 22px 40px minmax(0, 1.5fr) minmax(0, 1.3fr) 110px 100px 130px; }
  .more { display: block; margin: 10px auto 0; border: 0; border-radius: 6px; padding: 8px 16px; background: var(--raised); color: var(--t-head); font-weight: 600; cursor: pointer; }
  .more:hover { background: var(--hover); }
  .servers { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  .servers li {
    display: grid; grid-template-columns: var(--cols);
    align-items: center; gap: 14px; padding: 10px 8px; border-radius: 8px;
  }
  .servers li:hover { background: var(--hover); }
  .rk { color: var(--t-faint); font-weight: 700; text-align: right; font-size: 13px; }
  .who { min-width: 0; }
  .who strong { display: block; color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .who small { display: block; color: var(--t-muted); font-size: 12px; }
  .msgs { display: grid; gap: 6px; }
  .msgs b, .stat { color: var(--t-head); font-weight: 600; }

  .tags { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 10px; }
  .tags li { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 8px; background: var(--sunk); min-width: 0; }
  .tags span { color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
  .tags small { color: var(--t-muted); font-size: 12px; white-space: nowrap; }

  /* Phones show only the sorted column, under the name. */
  @media (max-width: 760px) {
    .servers li { grid-template-columns: 22px 40px 1fr; row-gap: 8px; }
    .msgs, .stat { display: none; grid-column: 3; }
    .msgs.on, .stat.on { display: grid; }
  }
</style>
