<script>
  import Avatar from "../components/Avatar.svelte";
  import ProfileCard from "../components/ProfileCard.svelte";
  import AliveBar from "../components/AliveBar.svelte";
  import AreaChart from "../components/AreaChart.svelte";
  import Heatmap from "../components/Heatmap.svelte";
  import Legend from "../components/Legend.svelte";
  import WeekPunch from "../components/WeekPunch.svelte";
  import { activity, fmt, fmtCompact, longDate, overview, yearStats, yearTotals } from "../lib/stats.js";

  let { result, view, range, games } = $props();

  const facts = $derived(result.facts);
  const profile = $derived(facts.profile);
  const sources = $derived(result.sources);
  const s = $derived(overview(view, facts, range));
  const totals = $derived(yearTotals(view));

  // The calendar follows the range when it's a year; on "All time" it has its own year tabs.
  let picked = $state(null);
  const year = $derived(range === "all" ? (picked ?? view.years.at(-1)) : range);
  const ys = $derived(yearStats(view, year));

  const dayKey = (ms) => new Date(ms).toISOString().slice(0, 10);
  const lastHour = $derived(facts.alive_hours.at(-1)?.[0] ?? 0);
  const fromKey = $derived(profile ? dayKey(profile.created_ms) : "");
  const untilKey = $derived(dayKey(sources?.packageDate || lastHour * 3_600_000));

  const act = $derived(activity(view, facts, range, sources?.packageDate || lastHour * 3_600_000));
  const hours = (n) => (n >= 10 ? fmt(n) : n.toFixed(1).replace(/\.0$/, ""));
  const pct = $derived(s.sent ? Math.round((s.alive / s.sent) * 100) : 0);
  const topMessages = $derived(s.people[0]?.messages ?? 1);

  const folder = (name) => sources?.folders?.[name];
  const folderNote = (f) => (!f ? "not in package" : f.read ? `${fmtCompact(f.events)} events` : "skipped");
  const analyticsNote = $derived(
    games.state === "reading" ? `reading ${Math.round((games.done / Math.max(games.total, 1)) * 100)}%`
      : games.state === "done" ? `${fmtCompact(games.events ?? 0)} events`
      : games.state === "failed" ? "couldn't read"
      : folder("analytics") ? "skipped" : "not in package",
  );
</script>

<div class="wrap">
  {#if profile}
    <ProfileCard {result} {games}>
      <dl class="headline">
        {#if facts.has_send_log}
          <div><dt>Messages sent</dt><dd class="num">{fmt(s.sent)}</dd></div>
          <div><dt>Still there</dt><dd class="num">{fmt(s.alive)}<small>{pct}%</small></dd></div>
        {:else}
          <div><dt>Messages</dt><dd class="num">{fmt(s.alive)}</dd></div>
        {/if}
        <div><dt>Hours in voice</dt><dd class="num">{facts.voice ? fmt(s.voiceHours) : "–"}</dd></div>
        <div><dt>Days active</dt><dd class="num">{fmt(s.daysActive)}</dd></div>
      </dl>
    </ProfileCard>
  {/if}

  <div class="grid">
    <section class="card span12">
      <div class="chead">
        <h2>Messages per day</h2>
        <Legend />
      </div>
      {#if range === "all"}
        <div class="ytabs" role="tablist" aria-label="Year">
          {#each view.years as y}
            <button class="ytab" role="tab" aria-selected={y === year} onclick={() => (picked = y)}>
              {y}<small class="num">{fmt(totals.get(y) ?? 0)}</small>
            </button>
          {/each}
        </div>
      {/if}
      <Heatmap daily={view.daily} breaks={view.breaks} {year} from={fromKey} until={untilKey} />
      <div class="calfoot">
        <span>{#if ys.best}Busiest day <b>{longDate(ys.best.key)}</b>, {fmt(ys.best.n)} messages{:else}No messages in {year}{/if}</span>
        <span>Active <b>{fmt(ys.days)}</b> days · longest streak <b>{fmt(ys.longest)} days</b></span>
      </div>
    </section>

    <section class="card span12">
      <div class="chead">
        <h2>Activity over time</h2>
        <div class="key">
          {#if facts.has_send_log}
            <span><i class="swatch alive"></i>Still there</span>
            <span><i class="swatch deleted"></i>Deleted</span>
            <span><i class="swatch lost"></i>Deleted server</span>
          {/if}
        </div>
      </div>
      <h3>Messages {act.weekly ? "per week" : "per month"}</h3>
      <AreaChart
        points={act.points}
        weekly={act.weekly}
        format={fmt}
        label="Messages {act.weekly ? 'per week' : 'per month'}"
        series={facts.has_send_log
          ? [{ key: "alive", label: "Still there", colour: "var(--acc)" }, { key: "deleted", label: "Deleted", colour: "var(--t-muted)" }, { key: "lost", label: "Deleted server", colour: "hatch" }]
          : [{ key: "alive", label: "Messages", colour: "var(--acc)" }]}
      />
      {#if facts.voice}
        <h3>Hours in voice {act.weekly ? "per week" : "per month"}</h3>
        <AreaChart
          points={act.points}
          weekly={act.weekly}
          height={160}
          format={hours}
          label="Hours in voice {act.weekly ? 'per week' : 'per month'}"
          series={[{ key: "voice", label: "Hours in voice", colour: "var(--acc-text)" }]}
        />
      {/if}
    </section>

    {#if facts.has_send_log}
      <section class="card span7">
        <h2>What's still there</h2>
        <AliveBar alive={s.alive} deleted={s.deleted} lost={s.lost} />
        <div class="rows">
          <div class="row"><i class="swatch alive"></i><span><b class="num">{fmt(s.alive)}</b> still there</span><span></span></div>
          <div class="row"><i class="swatch deleted"></i><span><b class="num">{fmt(s.deleted)}</b> deleted</span><span></span></div>
          <div class="row"><i class="swatch lost"></i><span><b class="num">{fmt(s.lost)}</b> deleted server</span><span></span></div>
        </div>
      </section>
    {/if}

    <section class="card {facts.has_send_log ? 'span5' : 'span12'}">
      <h2>Data sources</h2>
      <div class="src">
        <div><span class="ok" aria-hidden="true">✓</span><span><strong>Messages</strong><small>{fmt(sources?.channels ?? facts.channels.length)} channels</small></span><span class="n">{fmt(facts.alive_hours.reduce((n, b) => n + b[1], 0))} messages</span></div>
        {#each [["reporting", "Activity · reporting", "voice, servers, deleted messages"], ["tns", "Activity · trust & safety", "streams, edits, two-factor"]] as [id, label, what]}
          {@const f = folder(id)}
          <div>
            <span class={f?.read ? "ok" : "skip"} aria-hidden="true">{f?.read ? "✓" : ""}</span>
            <span><strong>{label}</strong><small>{what}</small></span>
            <span class="n">{folderNote(f)}</span>
          </div>
        {/each}
        <div>
          <span class={games.state === "done" ? "ok" : games.state === "reading" ? "busy" : "skip"} aria-hidden="true">{games.state === "done" ? "✓" : ""}</span>
          <span><strong>Activity · analytics</strong><small>games, notifications, searches</small></span>
          <span class="n">{analyticsNote}</span>
        </div>
      </div>
      {#if games.state === "none"}
        <p class="note">No analytics folder: games only show what you played while in voice.</p>
      {:else}
        <p class="note">Without analytics: no full game playtime, notifications, searches or crashes.</p>
      {/if}
    </section>

    <section class="card span7">
      <h2>When you talk <small>({view.tz.replace(/_/g, " ")} time)</small></h2>
      <WeekPunch grid={s.hours} unit="messages" label="Messages by weekday and hour of day" />
    </section>

    <section class="card span5">
      <h2>Your people <small>(top DMs)</small></h2>
      <ol class="people">
        {#each s.people as p, i (p.id)}
          <li>
            <span class="rk">{i + 1}</span>
            <Avatar id={p.id} hash={p.avatar} name={p.display_name} size={36} />
            <div>
              <strong>{p.display_name}</strong><small class="num">{fmt(p.messages)}</small>
              <div class="meter"><i style:width="{(p.messages / topMessages) * 100}%"></i></div>
            </div>
          </li>
        {:else}
          <li class="empty">No DMs in this range.</li>
        {/each}
      </ol>
    </section>
  </div>
</div>

<style>
  .wrap { max-width: 1320px; margin: 0 auto; padding: 24px; }

  .headline { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); border-top: 1px solid var(--line); margin: 0; }
  .headline div { padding: 18px 24px; }
  .headline div + div { border-left: 1px solid var(--line); }
  .headline dt { font-size: 14px; color: var(--t-muted); font-weight: 600; }
  .headline dd { margin: 4px 0 0; font: 900 28px/1.1 var(--display); color: var(--t-head); letter-spacing: -.01em; }
  .headline dd small { font: 600 14px var(--ui); color: var(--acc-text); margin-left: 6px; letter-spacing: 0; }

  .grid { display: grid; grid-template-columns: repeat(12, 1fr); gap: 16px; margin-top: 16px; }
  .span12 { grid-column: span 12; }
  .span7 { grid-column: span 7; }
  .span5 { grid-column: span 5; }

  .chead { display: flex; justify-content: space-between; align-items: start; gap: 16px; flex-wrap: wrap; }
  .ytabs { display: flex; gap: 18px; border-bottom: 1px solid var(--line); margin: 0 0 16px; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; }
  .ytab { padding: 6px 0 10px; border: 0; background: none; color: var(--t-muted); font-weight: 600; cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -1px; }
  .ytab:hover { color: var(--t-body); }
  .ytab[aria-selected="true"] { color: var(--t-head); border-bottom-color: var(--acc); }
  .ytab small { display: block; font-size: 12px; font-weight: 500; color: var(--t-faint); }
  .calfoot { display: flex; justify-content: space-between; gap: 16px; flex-wrap: wrap; margin-top: 12px; font-size: 14px; color: var(--t-muted); }
  .calfoot b { color: var(--t-head); }

  .rows { margin-top: 16px; display: grid; gap: 10px; }
  .row { display: grid; grid-template-columns: auto 1fr auto; align-items: baseline; gap: 10px; font-size: 15px; }
  .row b { color: var(--t-head); }
  .row span:last-child { color: var(--t-muted); font-size: 13px; }

  h3 { margin: 4px 0 10px; font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .key { display: flex; gap: 16px; flex-wrap: wrap; font-size: 13px; color: var(--t-muted); }
  .key span { display: flex; align-items: center; gap: 6px; }
  .busy { background: var(--raised); box-shadow: inset 0 0 0 3px var(--acc); animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: .4; } }

  .src > div { display: grid; grid-template-columns: 22px 1fr auto; gap: 10px; align-items: center; padding: 9px 0; border-bottom: 1px solid var(--line); font-size: 14px; }
  .src > div:last-child { border-bottom: 0; }
  .ok, .skip { width: 20px; height: 20px; border-radius: 50%; display: grid; place-items: center; font-size: 12px; font-weight: 800; }
  .ok { background: var(--green); color: #fff; }
  .skip { background: var(--raised); }
  .src strong { color: var(--t-head); font-weight: 600; display: block; }
  .src small { color: var(--t-muted); }
  .src .n { color: var(--t-muted); font-size: 13px; text-align: right; }
  .note { margin: 14px 0 0; padding: 12px 14px; border-radius: 8px; background: var(--acc-soft); font-size: 14px; color: var(--t-body); }

  .people { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; }
  .people li { display: grid; grid-template-columns: 22px 36px 1fr; align-items: center; gap: 10px; padding: 8px; border-radius: 8px; }
  .people li:hover { background: var(--hover); }
  .rk { color: var(--t-faint); font-weight: 700; text-align: right; }
  .people strong { color: var(--t-head); font-weight: 600; }
  .people small { color: var(--t-muted); font-size: 13px; float: right; }
  .meter { height: 6px; border-radius: 3px; background: var(--sunk); margin-top: 6px; }
  .meter i { display: block; height: 100%; border-radius: 3px; background: var(--acc); }
  .people .empty { display: block; color: var(--t-muted); }

  @media (max-width: 900px) {
    .span7, .span5 { grid-column: span 12; }
    .headline { grid-template-columns: repeat(2, 1fr); }
    .headline div:nth-child(odd) { border-left: 0; }
    .headline div:nth-child(n + 3) { border-top: 1px solid var(--line); }
  }
  @media (max-width: 620px) {
    .wrap { padding: 16px; }
    .headline div { padding: 14px 16px; }
    .headline dd { font-size: 22px; }
    .card { padding: 16px; }
  }
</style>
