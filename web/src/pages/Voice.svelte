<script>
  import AreaChart from "../components/AreaChart.svelte";
  import Avatar from "../components/Avatar.svelte";
  import RadialHours from "../components/RadialHours.svelte";
  import ServerIcon from "../components/ServerIcon.svelte";
  import WeekPunch from "../components/WeekPunch.svelte";
  import { serverInfo } from "../lib/discord.js";
  import { fmt, monthYear, shortDate, tnsStats, voiceStats } from "../lib/stats.js";

  let { result, view, range } = $props();

  const facts = $derived(result.facts);
  const s = $derived(voiceStats(view, facts, range));
  const streams = $derived(tnsStats(facts, range));
  const hours = (n) => (n >= 100 ? fmt(n) : n >= 10 ? n.toFixed(0) : n.toFixed(1).replace(/\.0$/, ""));
  const duration = (h) => (h >= 1 ? `${Math.floor(h)} h ${Math.round((h % 1) * 60)} min` : `${Math.round(h * 60)} min`);
  const SERIES = [
    { key: "servers", label: "Servers", colour: "var(--cat-1)" },
    { key: "dms", label: "DM calls", colour: "var(--cat-2)" },
    { key: "groups", label: "Group calls", colour: "var(--cat-3)" },
  ];

  const servers = $derived(new Map(facts.servers.map((x) => [x.id, x])));
  let icons = $state({});
  const requested = new Set();
  $effect(() => {
    for (const p of s?.places.slice(0, 10) ?? []) {
      const info = servers.get(p.id);
      if (p.kind !== "servers" || requested.has(p.id) || !info) continue;
      requested.add(p.id);
      serverInfo(info, result.icons?.[p.id]).then((i) => (icons = { ...icons, [p.id]: i }));
    }
  });
</script>

<div class="page">
  {#if !s}
    <section class="card"><h2>Voice</h2><p class="muted">Your package has no voice activity.</p></section>
  {:else}
    <section class="strip">
      <dl>
        <div><dt>Hours in voice</dt><dd>{fmt(s.hours)}</dd></div>
        <div><dt>Sessions</dt><dd>{fmt(s.sessions)}</dd></div>
        <div><dt>Average session</dt><dd>{duration(s.avg)}</dd></div>
        <div><dt>Longest session</dt><dd>{s.longest[0] ? `${s.longest[0].hours.toFixed(1)} h` : "–"}</dd></div>
        <div><dt>Days in voice</dt><dd>{fmt(s.days)}</dd></div>
        {#if streams}
          <div data-tip="Since {monthYear(streams.since)}"><dt>Hours streamed</dt><dd>{fmt(streams.streamed)}</dd></div>
          <div data-tip="Since {monthYear(streams.since)}"><dt>Watching streams</dt><dd>{fmt(streams.watched)}<small>h</small></dd></div>
        {/if}
      </dl>
    </section>

    <div class="grid12">
      {#if s.points.length > 1}
        <section class="card s12">
          <div class="chead">
            <h2>Hours per month</h2>
            <div class="key">{#each SERIES as x (x.key)}<span><i style:background={x.colour}></i>{x.label}</span>{/each}</div>
          </div>
          <AreaChart points={s.points} series={SERIES} format={hours} label="Hours in voice per month, by where" />
        </section>
      {/if}

      <section class="card s5">
        <h2>Time of day <small>({view.tz.replace(/_/g, " ")} time)</small></h2>
        <RadialHours hours={s.clock} format={hours} unit=" hours" label="Hours in voice by hour of day" />
      </section>

      <section class="card s7">
        <h2>By hour and weekday</h2>
        <WeekPunch grid={s.grid} format={hours} unit="hours" label="Hours in voice by weekday and hour of day" />
        <div class="split">
          {#each SERIES as x, i (x.key)}
            <div><i style:background={x.colour}></i><b class="num">{fmt(s.split[i])}</b> h {x.label.toLowerCase()}</div>
          {/each}
        </div>
      </section>

      <section class="card s7">
        <h2>Where you talk</h2>
        <ol class="places">
          {#each s.places.slice(0, 10) as p (p.id)}
            <li>
              {#if p.kind === "dms" && p.person}
                <Avatar id={p.person.id} hash={p.person.avatar} name={p.person.display_name} size={36} />
              {:else}
                <ServerIcon url={icons[p.id]?.icon} name={p.name} size={36} />
              {/if}
              <div>
                <strong>{p.name}</strong>
                <span class="meter"><i style:width="{(p.hours / s.places[0].hours) * 100}%"></i></span>
              </div>
              <span class="n num">{hours(p.hours)} h</span>
            </li>
          {/each}
        </ol>
      </section>

      <section class="card s5">
        <h2>Longest sessions</h2>
        <ol class="long">
          {#each s.longest.slice(0, 8) as x (x.start_ms)}
            <li>
              <b class="num">{duration(x.hours)}</b>
              <div><strong>{x.name}</strong><small>{shortDate(x.start_ms)}</small></div>
            </li>
          {:else}
            <li class="muted">No sessions in this range.</li>
          {/each}
        </ol>
      </section>
    </div>
  {/if}
</div>

<style>
  .key { display: flex; gap: 14px; flex-wrap: wrap; font-size: 13px; color: var(--t-muted); }
  .key span { display: flex; align-items: center; gap: 6px; }
  .key i, .split i { width: 10px; height: 10px; border-radius: 3px; display: inline-block; }
  .split { display: flex; gap: 18px; flex-wrap: wrap; margin-top: 18px; font-size: 14px; color: var(--t-muted); }
  .split div { display: flex; align-items: center; gap: 6px; }
  .split b { color: var(--t-head); }

  .places { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
  .places li { display: grid; grid-template-columns: 36px 1fr auto; align-items: center; gap: 12px; }
  .places div { min-width: 0; }
  .places strong { display: block; color: var(--t-head); font-weight: 600; margin-bottom: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .n { color: var(--t-muted); font-size: 13px; }

  .long { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .long li { display: grid; grid-template-columns: 100px 1fr; gap: 12px; align-items: baseline; }
  .long b { color: var(--acc-text); font-weight: 700; }
  .long div { min-width: 0; }
  .long strong { display: block; color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .long small { color: var(--t-muted); font-size: 12px; }
</style>
