<script>
  import AreaChart from "../components/AreaChart.svelte";
  import Avatar from "../components/Avatar.svelte";
  import DivergingBars from "../components/DivergingBars.svelte";
  import SortHeads from "../components/SortHeads.svelte";
  import TagChip from "../components/TagChip.svelte";
  import { fmt, friendCounts, monthYear, peopleStats, shortDate } from "../lib/stats.js";

  let { result, view, range } = $props();

  const facts = $derived(result.facts);
  const profile = $derived(facts.profile);
  const s = $derived(peopleStats(view, facts, range));
  const friends = $derived(friendCounts(facts, range));

  // Sortable columns. First message starts oldest first, the rest biggest first; people without
  // a value for the column go last either way.
  const COLUMNS = [
    { key: "messages", label: "Messages", value: (p) => p.messages },
    { key: "voice", label: "Hours in calls", value: (p) => p.voice },
    { key: "first", label: "First message", value: (p) => (p.first < Infinity ? p.first : 0), asc: true },
  ];
  let sort = $state("messages");
  let desc = $state(true);
  // The top 15, or everyone in a scrolling list.
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
  const max = $derived(Math.max(1, ...s.ranked.map((p) => p.messages)));
  const hours = (n) => (n >= 10 ? `${fmt(n)} h` : n >= 0.05 ? `${n.toFixed(1)} h` : "–");
</script>

<div class="page">
  <section class="strip">
    <dl>
      <div><dt>Friends</dt><dd>{fmt(profile?.friends ?? 0)}</dd></div>
      <div><dt>People you DM'd</dt><dd>{fmt(s.dms)}</dd></div>
      <div><dt>Group chats</dt><dd>{fmt(s.groups.length)}</dd></div>
      <div><dt>New friends</dt><dd>{fmt(s.added)}</dd></div>
      <div><dt>Removed or declined</dt><dd>{fmt(s.removed)}</dd></div>
      <div><dt>Blocked</dt><dd>{fmt(profile?.blocked ?? 0)}</dd></div>
    </dl>
  </section>

  <div class="grid12">
    <section class="card s12 list">
      <SortHeads title="Your people" columns={COLUMNS} bind:sort bind:desc onsort={() => list?.scrollTo(0, 0)} />
      <ol class="people" class:scroll-list={all} bind:this={list}>
        {#each top as p, i (p.id)}
          <li>
            <span class="rk">{i + 1}</span>
            <Avatar id={p.id} hash={p.avatar} name={p.display_name} size={40} decoration={p.decoration} />
            <div class="who">
              <div class="name">
                <strong>{p.display_name}</strong>
                {#if p.tag}<TagChip tag={p.tag.tag} guild={p.tag.guild} badgeHash={p.tag.badge} />{/if}
              </div>
              <small>@{p.username}{p.nickname ? ` · you call them “${p.nickname}”` : ""}</small>
            </div>
            <div class="msgs" class:on={sort === "messages"}>
              <b class="num">{fmt(p.messages)}</b>
              <span class="meter"><i style:width="{(p.messages / max) * 100}%"></i></span>
            </div>
            <b class="stat num" class:on={sort === "voice"}>{hours(p.voice)}</b>
            <b class="stat" class:on={sort === "first"}>{p.first < Infinity ? monthYear(p.first) : "–"}</b>
          </li>
        {:else}
          <li class="muted">No DMs in this range.</li>
        {/each}
      </ol>
      {#if sorted.length > 15}
        <button class="more" onclick={() => (all = !all)}>{all ? "Show fewer" : `Show all ${fmt(sorted.length)}`}</button>
      {/if}
    </section>

    {#if s.points.length > 1}
      <section class="card s12">
        <div class="chead">
          <h2>Who you talk to</h2>
          <div class="key">
            {#each s.series as x (x.key)}<span><i style:background={x.colour}></i>{x.label}</span>{/each}
          </div>
        </div>
        <AreaChart points={s.points} series={s.series} format={fmt} label="DM messages per month, by person" />
      </section>
    {/if}

    {#if friends.length > 1}
      <section class="card s12">
        <h2>Friends you had <small>(logged since {monthYear(facts.friend_counts[0][0])})</small></h2>
        <AreaChart points={friends} series={[{ key: "friends", label: "Friends", colour: "var(--acc)" }]} height={200} format={fmt} label="Friends you had, per month" />
      </section>
    {/if}

    <section class="card s7">
      <h2>Friend list changes <small>(the log doesn't say who)</small></h2>
      {#if s.friendYears.length}
        <DivergingBars points={s.friendYears} upLabel="New friends" downLabel="Removed or declined" />
      {:else}
        <p class="muted">No friend list changes in this range.</p>
      {/if}
    </section>

    <section class="card s5">
      <h2>Group chats</h2>
      <ol class="rank">
        {#each s.groups.slice(0, 8) as g (g.id)}
          <li>
            <div><strong>{g.name}</strong><small>{shortDate(g.first)} – {shortDate(g.last)}</small></div>
            <span class="n num">{fmt(g.n)}</span>
            <span class="meter"><i style:width="{(g.n / s.groups[0].n) * 100}%"></i></span>
          </li>
        {:else}
          <li class="muted">No group chats in this range.</li>
        {/each}
      </ol>
    </section>
  </div>
</div>

<style>
  .list { --cols: 22px 40px minmax(0, 1.5fr) minmax(0, 1.3fr) 110px 130px; }
  .more { display: block; margin: 10px auto 0; border: 0; border-radius: 6px; padding: 8px 16px; background: var(--raised); color: var(--t-head); font-weight: 600; cursor: pointer; }
  .more:hover { background: var(--hover); }
  .people { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  .people li {
    display: grid; grid-template-columns: var(--cols);
    align-items: center; gap: 14px; padding: 10px 8px; border-radius: 8px;
  }
  .people li:hover { background: var(--hover); }
  .rk { color: var(--t-faint); font-weight: 700; text-align: right; font-size: 13px; }
  .who { min-width: 0; }
  .name { display: flex; align-items: center; gap: 6px; min-width: 0; }
  .name strong { color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .who small { display: block; color: var(--t-muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .msgs { display: grid; gap: 6px; }
  .msgs b, .stat { color: var(--t-head); font-weight: 600; }

  .key { display: flex; gap: 14px; flex-wrap: wrap; font-size: 13px; color: var(--t-muted); }
  .key span { display: flex; align-items: center; gap: 6px; }
  .key i { width: 10px; height: 10px; border-radius: 3px; }

  .rank { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .rank li { display: grid; grid-template-columns: 1fr auto; gap: 4px 12px; align-items: end; }
  .rank li .meter { grid-column: 1 / -1; }
  .rank div { min-width: 0; }
  .rank strong { color: var(--t-head); font-weight: 600; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rank small { color: var(--t-muted); font-size: 12px; }
  .n { color: var(--t-muted); font-size: 13px; }

  /* Phones show only the sorted column, under the name. */
  @media (max-width: 760px) {
    .people li { grid-template-columns: 22px 40px 1fr; row-gap: 8px; }
    .msgs, .stat { display: none; grid-column: 3; }
    .msgs.on, .stat.on { display: grid; }
  }
</style>
