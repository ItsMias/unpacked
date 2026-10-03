<script>
  import { onMount } from "svelte";
  import TopBar from "./components/TopBar.svelte";
  import Home from "./pages/Home.svelte";
  import Loading from "./pages/Loading.svelte";
  import Overview from "./pages/Overview.svelte";
  import Messages from "./pages/Messages.svelte";
  import People from "./pages/People.svelte";
  import Servers from "./pages/Servers.svelte";
  import Voice from "./pages/Voice.svelte";
  import Timeline from "./pages/Timeline.svelte";
  import Games from "./pages/Games.svelte";
  import Expressions from "./pages/Expressions.svelte";
  import Devices from "./pages/Devices.svelte";
  import Purchases from "./pages/Purchases.svelte";
  import Privacy from "./pages/Privacy.svelte";
  import Stats from "./pages/Stats.svelte";
  import { buildView, fmt, yearTotals } from "./lib/stats.js";

  const PAGES = [
    { id: "overview", label: "Overview", component: Overview },
    { id: "messages", label: "Messages", component: Messages },
    { id: "people", label: "People", component: People },
    { id: "servers", label: "Servers", component: Servers },
    { id: "voice", label: "Voice", component: Voice },
    { id: "games", label: "Games", component: Games },
    { id: "timeline", label: "Timeline", component: Timeline, more: true },
    { id: "stats", label: "Statistics", component: Stats, more: true },
    { id: "emoji", label: "Emoji & GIFs", component: Expressions, more: true },
    { id: "devices", label: "Devices", component: Devices, more: true },
    { id: "purchases", label: "Purchases", component: Purchases, more: true },
    { id: "privacy", label: "What Discord knows", component: Privacy, more: true },
  ];

  let phase = $state("drop"); // drop | loading | ready | error
  let error = $state("");
  // What the workers have reported while loading, for the loading screen.
  let load = $state({});
  let result = $state.raw(null);
  // Game history and server tags load in their own worker, alongside and usually after the rest.
  // state: idle | reading | done | none (no analytics folder) | failed
  let games = $state.raw({ state: "idle" });
  let range = $state("all");
  let page = $state("overview");
  // An address without a page (/ rather than /#/messages) is the home page, also while a package
  // is open: the logo leads there, and Back or the top bar's button returns to the stats.
  let home = $state(true);
  // Made-up data from the "Try the demo" button (or ?demo in the URL).
  let demo = $state(false);

  const tz = $derived(result?.facts.time_zone ?? Intl.DateTimeFormat().resolvedOptions().timeZone);
  const view = $derived(result ? buildView(result.facts, tz) : null);
  const ranges = $derived.by(() => {
    if (!view) return [];
    const totals = yearTotals(view);
    const all = [...totals.values()].reduce((a, b) => a + b, 0);
    return [
      { value: "all", label: "All time", note: fmt(all) },
      ...[...view.years].reverse().map((y) => ({ value: y, label: String(y), note: fmt(totals.get(y) ?? 0) })),
    ];
  });
  const status = $derived(
    phase === "ready" && games.state === "reading" ? `Reading game history ${Math.round(((games.done ?? 0) / Math.max(games.total ?? 1, 1)) * 100)}%` : "",
  );

  let workers = [];
  // The wait on the finished loading screen, cancelled when the logo leads home meanwhile.
  let reveal = 0;

  /** [key, value, key, value, …] from a worker into a Map. */
  function pairs(flat) {
    const m = new Map();
    for (let i = 0; i < flat.length; i += 2) m.set(flat[i], flat[i + 1]);
    return m;
  }

  function start(file, job, onUpdate, onDone, onError) {
    const w = new Worker(new URL("./lib/worker.js", import.meta.url), { type: "module" });
    w.onmessage = ({ data }) => {
      if (data.type === "done") { onDone(data.result); w.terminate(); }
      else if (data.type === "error") onError(data.message);
      else onUpdate(data);
    };
    w.onerror = (e) => onError(e.message || "Something went wrong while reading the package.");
    w.postMessage({ file, job });
    workers.push(w);
  }

  function open(file) {
    workers.forEach((w) => w.terminate());
    workers = [];
    demo = false;
    phase = "loading";
    load = { started: Date.now(), bytes: file.size, days: new Map(), voice: new Map(), deleted: new Map(), tns: { state: "reading" } };
    games = { state: "reading" };
    range = "all";
    // The trust & safety worker usually finishes first; its part is folded into the facts once
    // the main result is in.
    let main = null, extra = null;
    const merge = () => (result = extra ? withTns(main, extra) : main);

    start(file, "main",
      (u) => {
        if (u.type === "start") Object.assign(load, { packageDate: u.packageDate, channels: u.channels, hasAnalytics: u.hasAnalytics });
        else if (u.type === "account") load.account = u;
        else if (u.type === "progress") {
          load.progress = u;
          if (u.days) load.days = pairs(u.days);
          if (u.activity) {
            const voice = new Map(), deleted = new Map();
            for (let i = 0; i < u.activity.length; i += 3) {
              if (u.activity[i + 1]) voice.set(u.activity[i], u.activity[i + 1]);
              if (u.activity[i + 2]) deleted.set(u.activity[i], u.activity[i + 2]);
            }
            Object.assign(load, { voice, deleted });
          }
        }
        if (u.type === "start" && !u.hasAnalytics) games = { state: "none" };
      },
      (r) => {
        main = r;
        merge();
        load.progress = { ...load.progress, stage: "done" };
        // A moment on the finished calendar before the page takes over.
        reveal = setTimeout(showStats, Date.now() - load.started > 3000 ? 900 : 0);
      },
      (message) => {
        error = message;
        phase = "error";
        workers.forEach((w) => w.terminate());
      });
    start(file, "games",
      (u) => {
        if (u.type === "progress" && games.state === "reading") {
          games = { state: "reading", done: u.done, total: u.total, counts: u.counts, days: u.days ? pairs(u.days) : games.days };
        }
      },
      (r) => (games = r ? { state: "done", ...r } : { state: "none" }),
      () => (games = { state: "failed" }));
    start(file, "tns",
      (u) => { if (u.type === "progress") load.tns = { state: "reading", done: u.done, total: u.total, events: u.counts?.events ?? 0 }; },
      (t) => {
        load.tns = { state: t ? "done" : "none" };
        if (!t) return;
        extra = t;
        if (main) merge();
      },
      () => (load.tns = { state: "failed" }));
  }

  /** The stats of a package that's been read, on the Overview unless the address names a page. */
  function showStats() {
    phase = "ready";
    if (home) {
      location.hash = "#/overview";
      // Now rather than on hashchange, so the home page doesn't flash up first.
      route();
    }
  }

  /** The main result with what the trust & safety worker found added to its facts and sources. */
  function withTns(r, t) {
    const folders = r.sources?.folders ?? {};
    return {
      ...r,
      facts: { ...r.facts, tns: t.tns ?? r.facts.tns },
      sources: { ...r.sources, folders: { ...folders, tns: { size: folders.tns?.size ?? 0, read: true, events: t.events, types: t.types } } },
    };
  }

  /**
   * Opens the made-up demo account, on page `to`. `?demo` goes in the URL so a reload keeps it; from
   * the home page it's a new history entry, so Back returns there.
   */
  async function openDemo(to = "overview", push = phase === "drop") {
    workers.forEach((w) => w.terminate());
    workers = [];
    const { demoResult } = await import("./lib/demo.js");
    const d = demoResult();
    result = d.result;
    games = d.games;
    range = "all";
    demo = true;
    phase = "ready";
    history[push ? "pushState" : "replaceState"](null, "", `${location.pathname}?demo#/${to}`);
    route();
  }

  // Back and forward between the home page and the demo.
  function onHistory() {
    const wantDemo = new URLSearchParams(location.search).has("demo");
    if (demo && !wantDemo) reset();
    else if (!demo && wantDemo && phase === "drop") openDemo(page, false);
  }

  function reset() {
    workers.forEach((w) => w.terminate());
    clearTimeout(reveal);
    result = null;
    demo = false;
    phase = "drop";
    const params = new URLSearchParams(location.search);
    params.delete("demo");
    history.replaceState(null, "", location.pathname + (params.size ? `?${params}` : ""));
    route();
  }

  /**
   * The logo: the home page. A package that's been read stays open, so Back or the top bar's
   * button returns to it; the demo and a package still being read are closed.
   */
  function goHome() {
    if (phase === "ready" && !demo) {
      if (!home) history.pushState(null, "", location.pathname + location.search);
      return route();
    }
    // Leaving the demo is a new history entry too, so Back reopens it.
    if (demo) history.pushState(null, "", location.pathname);
    reset();
  }

  const route = () => {
    const id = location.hash.replace(/^#\/?/, "");
    home = !id;
    // On the home page `page` keeps the last one, for the way back.
    if (id) page = PAGES.some((p) => p.id === id) ? id : "overview";
    window.scrollTo(0, 0);
  };

  // One tooltip for every element with data-tip.
  let tip = $state({ on: false, x: 0, y: 0, text: "", sub: "" });
  function onPointer(e) {
    const t = e.target.closest?.("[data-tip]");
    if (!t) return void (tip.on = false);
    const r = t.getBoundingClientRect();
    tip = { on: true, x: r.left + r.width / 2, y: r.top, text: t.dataset.tip, sub: t.dataset.sub ?? "" };
  }

  onMount(async () => {
    route();
    if (new URLSearchParams(location.search).has("demo")) return openDemo(page, false);
    // Development shortcut: ?facts=/dev/facts.json loads saved engine output instead of a zip.
    const devFacts = import.meta.env.DEV && new URLSearchParams(location.search).get("facts");
    if (devFacts) {
      const facts = await (await fetch(devFacts)).json();
      result = { facts, avatar: null, avatars: [], icons: {}, sources: null, hasAnalytics: facts.games?.source === "analytics" };
      games = result.hasAnalytics ? { state: "done", games: facts.games, tags: [] } : { state: "none" };
      showStats();
    }
  });

  const Page = $derived(PAGES.find((p) => p.id === page).component);
</script>

<svelte:window onhashchange={route} onpopstate={onHistory} onmouseover={onPointer} onscroll={() => (tip.on = false)} />

{#if phase === "ready" && view && !home}
  <TopBar pages={PAGES} {page} {ranges} bind:range {status} {demo} onopen={reset} onhome={goHome} />
  <Page {result} {view} {range} {games} />
{:else if phase === "loading"}
  <TopBar onhome={goHome} />
  <Loading {load} {games} />
{:else}
  <TopBar onhome={goHome} onback={phase === "ready" ? () => (location.hash = `#/${page}`) : null} />
  <Home {phase} {error} onfile={open} ondemo={openDemo} />
{/if}

<div class="tip" class:on={tip.on} style:left="{tip.x}px" style:top="{tip.y}px" role="tooltip">
  {tip.text}{#if tip.sub}<small>{tip.sub}</small>{/if}
</div>
