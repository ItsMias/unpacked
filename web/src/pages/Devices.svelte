<script>
  import AreaChart from "../components/AreaChart.svelte";
  import Donut from "../components/Donut.svelte";
  import RadialHours from "../components/RadialHours.svelte";
  import { deviceName, PLATFORM_LABELS } from "../lib/names.js";
  import { deviceStats, fmt, monthYear } from "../lib/stats.js";

  let { result, view, range } = $props();

  const s = $derived(deviceStats(view, result.facts, range));
  const SERIES = [
    { key: "android", label: "Android app", colour: "var(--cat-1)" },
    { key: "desktop", label: "Desktop app", colour: "var(--cat-2)" },
    { key: "web", label: "Browser", colour: "var(--cat-3)" },
    { key: "ios", label: "iOS app", colour: "var(--cat-4)" },
    { key: "other", label: "Other", colour: "var(--t-faint)" },
  ];
  const label = (c) => (c.device ? deviceName(c.device) : c.platform === "desktop" ? `Discord on ${c.os || "desktop"}` : `${c.browser}${c.os ? ` on ${c.os}` : ""}`);
  const ICON = {
    android: "M7 3h10a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM11 18h2",
    ios: "M7 3h10a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM11 18h2",
    desktop: "M3 5h18v11H3zM8 20h8M12 16v4",
    web: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9S9.5 5.6 12 3z",
    other: "M4 7h16v10H4zM8 11h.01M16 11h.01",
  };
  const OPENED = { launcher: "App icon", notification: "A notification", deeplink: "A link" };
  const opened = $derived(
    (s?.opened_from ?? []).slice(0, 4).map(([k, n], i) => ({ label: OPENED[k] ?? k.replace(/_/g, " "), value: n, colour: `var(--cat-${i + 1})` })),
  );
  const max = $derived(Math.max(1, ...(s?.clients ?? []).map((c) => c.count)));
</script>

<div class="page">
  {#if !s}
    <section class="card"><h2>Devices</h2><p class="muted">Your package has no session events.</p></section>
  {:else}
    <section class="strip">
      <dl>
        <div><dt>Devices and apps</dt><dd>{fmt(s.clients.length)}</dd></div>
        <div><dt>Phones and tablets</dt><dd>{fmt(s.clients.filter((c) => c.device).length)}</dd></div>
        <div><dt>App opens</dt><dd>{fmt(s.opens)}</dd></div>
        {#if opened.length}<div><dt>From a notification</dt><dd>{fmt(s.opened_from.find((o) => o[0] === "notification")?.[1] ?? 0)}</dd></div>{/if}
      </dl>
    </section>

    <div class="grid12">
      <section class="card s12">
        <h2>Your devices</h2>
        <ol class="clients">
          {#each s.clients.slice(0, 14) as c (c.platform + c.device + c.browser + c.os)}
            <li>
              <span class="ic" aria-hidden="true">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d={ICON[c.platform]} /></svg>
              </span>
              <div class="who">
                <strong>{label(c)}</strong>
                <small>{PLATFORM_LABELS[c.platform]}{c.device ? ` · ${c.device.split(",")[0]}` : ""}</small>
              </div>
              <div class="bar">
                <span class="meter"><i style:width="{(c.count / max) * 100}%"></i></span>
                <small class="num">{fmt(c.count)} sessions</small>
              </div>
              <span class="when">{monthYear(c.first_ms)} – {monthYear(c.last_ms)}</span>
            </li>
          {/each}
        </ol>
      </section>

      {#if s.points.length > 1}
        <section class="card s12">
          <div class="chead">
            <h2>How you open Discord</h2>
            <div class="key">{#each SERIES as x (x.key)}<span><i style:background={x.colour}></i>{x.label}</span>{/each}</div>
          </div>
          <AreaChart points={s.points} series={SERIES} format={fmt} label="Sessions per month by app" />
        </section>
      {/if}

      <section class="card s6">
        <h2>When you open the app <small>({view.tz.replace(/_/g, " ")} time)</small></h2>
        <RadialHours hours={s.clock.clock} format={fmt} unit=" opens" label="App opens by hour of day" />
      </section>

      {#if opened.length}
        <section class="card s6">
          <h2>What opened it</h2>
          <Donut segments={opened} centre={fmt(opened.reduce((a, o) => a + o.value, 0))} centreSub="opens" label="What you opened Discord from" />
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .clients { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  .clients li { display: grid; grid-template-columns: 40px minmax(0, 1.4fr) minmax(0, 1.2fr) 190px; align-items: center; gap: 14px; padding: 10px 8px; border-radius: 8px; }
  .clients li:hover { background: var(--hover); }
  .ic { width: 40px; height: 40px; border-radius: 12px; background: var(--raised); color: var(--acc-text); display: grid; place-items: center; }
  .who { min-width: 0; }
  .who strong { display: block; color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .who small, .bar small { display: block; color: var(--t-muted); font-size: 12px; }
  .bar small { margin-top: 4px; }
  .when { color: var(--t-muted); font-size: 13px; text-align: right; }
  .key { display: flex; gap: 14px; flex-wrap: wrap; font-size: 13px; color: var(--t-muted); }
  .key span { display: flex; align-items: center; gap: 6px; }
  .key i { width: 10px; height: 10px; border-radius: 3px; }
  @media (max-width: 760px) {
    .clients li { grid-template-columns: 40px 1fr; }
    .bar { grid-column: 2; }
    .when { display: none; }
  }
</style>
