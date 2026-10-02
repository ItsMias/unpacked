<script>
  import HistoryWall from "../components/HistoryWall.svelte";
  import { fmt, fmtCompact } from "../lib/stats.js";

  /** `load`: what the main worker has reported so far; `games`: the analytics worker (see App.svelte). */
  let { load, games } = $props();

  let now = $state(Date.now());
  $effect(() => {
    const t = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(t);
  });

  const p = $derived(load.progress);
  const c = $derived(p?.counts ?? {});
  const stage = $derived(p?.stage ?? "account");
  const order = ["account", "messages", "activity", "done"];
  const stepState = (s) => {
    const at = order.indexOf(stage), me = order.indexOf(s);
    return me < at ? "done" : me === at ? "active" : "pending";
  };
  const pct = (a, b) => `${Math.round((a / Math.max(b, 1)) * 100)}%`;
  const elapsed = $derived.by(() => {
    const s = Math.max(0, Math.round((now - load.started) / 1000));
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  });
  // Binary gigabytes, like file managers show them.
  const gb = (n) => (n / 2 ** 30).toFixed(1);
  const monthYear = (ms) => new Date(ms).toLocaleDateString("en-GB", { month: "short", year: "numeric", timeZone: "UTC" });

  const NONE = new Map();
  const tnsStep = $derived(
    { none: "skipped", failed: "skipped", done: "done" }[load.tns?.state] ?? (load.tns?.done ? "active" : "pending"),
  );

  const avatarUrl = $derived(load.account?.avatar ? URL.createObjectURL(load.account.avatar) : null);
  const years = $derived.by(() => {
    const last = new Date(load.packageDate || Date.now()).getUTCFullYear();
    const first = load.account ? new Date(load.account.created).getUTCFullYear() : last - 4;
    return [Math.max(2015, Math.min(first, last)), last];
  });
</script>

<main class="wrap">
  <section class="panel" aria-live="polite">
    <header>
      {#if avatarUrl}<img src={avatarUrl} alt="" width="56" height="56" />{:else}<span class="ph"></span>{/if}
      <div>
        <h1>{load.account ? `Unpacking ${load.account.name}` : "Opening your package"}</h1>
        <p>{gb(load.bytes ?? 0)} GB{load.packageDate ? ` · ${monthYear(load.packageDate)}` : ""}</p>
      </div>
    </header>

    <ol class="steps">
      <li class={stepState("account")}>
        <span class="dot" aria-hidden="true"></span>
        <div>
          <strong>Account</strong>
          <small>{load.account ? `${load.account.name} · on Discord since ${monthYear(load.account.created)}` : "Reading your profile"}</small>
        </div>
      </li>
      <li class={stepState("messages")}>
        <span class="dot" aria-hidden="true"></span>
        <div>
          <strong>Messages</strong>
          <small>
            {#if c.messages}{fmt(c.messages)} messages · {fmt(c.channelsDone ?? 0)} of {fmt(c.channelsTotal ?? 0)} chats{:else}{fmt(load.channels ?? 0)} chats to read{/if}
          </small>
          {#if stepState("messages") === "active"}<i class="bar"><i style:width={pct(c.channelsDone ?? 0, c.channelsTotal ?? 1)}></i></i>{/if}
        </div>
      </li>
      <li class={stepState("activity")}>
        <span class="dot" aria-hidden="true"></span>
        <div>
          <strong>Activity log</strong>
          <small>
            {#if c.events}{fmtCompact(c.events)} events · {fmt((c.deleted ?? 0) + (c.lost ?? 0))} deleted messages · {fmt(c.voice ?? 0)} voice sessions{:else}Voice, servers, deleted messages{/if}
          </small>
          {#if stepState("activity") === "active"}<i class="bar"><i style:width={pct(p.done, p.total)}></i></i>{/if}
        </div>
      </li>
      <li class={tnsStep}>
        <span class="dot" aria-hidden="true"></span>
        <div>
          <strong>Trust &amp; safety log</strong>
          <small>
            {#if load.tns?.state === "none"}Not in your package
            {:else if load.tns?.state === "done"}Done
            {:else if load.tns?.state === "failed"}Couldn't read it
            {:else if load.tns?.done}{fmtCompact(load.tns.events)} events · {pct(load.tns.done, load.tns.total)} · running alongside
            {:else}Streams, edits, two-factor · runs alongside{/if}
          </small>
          {#if tnsStep === "active"}<i class="bar"><i style:width={pct(load.tns.done, load.tns.total)}></i></i>{/if}
        </div>
      </li>
      <li class={load.hasAnalytics === false ? "skipped" : games?.state === "done" ? "done" : games?.done ? "active" : "pending"}>
        <span class="dot" aria-hidden="true"></span>
        <div>
          <strong>Game history</strong>
          <small>
            {#if load.hasAnalytics === false}Not in your package
            {:else if games?.state === "done"}Done
            {:else if games?.done}{fmtCompact(games.counts?.events ?? 0)} events · {pct(games.done, games.total)} · running alongside
            {:else}Runs alongside the rest{/if}
          </small>
          {#if games?.done && games.state !== "done"}<i class="bar"><i style:width={pct(games.done, games.total)}></i></i>{/if}
        </div>
      </li>
    </ol>

    <footer class="num">
      <span><b>{p?.total ? pct(p.done, p.total) : "0%"}</b> done</span>
      <span>{elapsed} elapsed</span>
    </footer>
  </section>

  <section class="canvas" aria-label="Your messages per day, filling in as they're read">
    <HistoryWall days={load.days} voice={load.voice} deleted={load.deleted} played={games?.days ?? NONE} {years} />
    <ul class="key" aria-hidden="true">
      <li class:on={load.days.size}><i class="msg"></i>Messages</li>
      <li class:on={load.voice.size}><i class="voice"></i>Voice</li>
      <li class:on={load.deleted.size}><i class="deleted"></i>Deleted messages</li>
      <li class:on={games?.days?.size}><i class="game"></i>Games</li>
    </ul>
  </section>
</main>

<style>
  .wrap {
    max-width: 1320px; margin: 0 auto; padding: 32px 24px; min-height: calc(100vh - 61px);
    display: grid; grid-template-columns: 380px 1fr; gap: 32px; align-items: center;
  }
  .panel { background: var(--card); border-radius: var(--radius); padding: 24px; }
  header { display: flex; align-items: center; gap: 14px; margin-bottom: 24px; }
  header img, .ph { width: 56px; height: 56px; border-radius: 50%; flex: none; background: var(--raised); }
  h1 { margin: 0; font: 800 20px/1.2 var(--ui); color: var(--t-head); }
  header p { margin: 2px 0 0; color: var(--t-muted); font-size: 14px; }

  .steps { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; }
  .steps li { display: grid; grid-template-columns: 20px 1fr; gap: 12px; padding: 10px 0; }
  .steps strong { display: block; color: var(--t-head); font-weight: 600; }
  .steps small { display: block; color: var(--t-muted); font-size: 13px; margin-top: 2px; }
  .pending strong, .skipped strong { color: var(--t-muted); }

  .dot { width: 20px; height: 20px; border-radius: 50%; margin-top: 1px; box-sizing: border-box; border: 2px solid var(--line); position: relative; }
  .done .dot { border: 0; background: var(--green) url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='3.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M5 12.5 10 17l9-10'/%3E%3C/svg%3E") center / 12px no-repeat; }
  .active .dot { border-color: var(--raised); border-top-color: var(--acc); animation: spin .8s linear infinite; }
  .skipped .dot { border-style: dashed; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .bar { display: block; height: 4px; border-radius: 2px; background: var(--sunk); margin-top: 8px; overflow: hidden; }
  .bar i { display: block; height: 100%; background: var(--acc); border-radius: 2px; transition: width .3s; }

  footer {
    display: flex; justify-content: space-between; align-items: baseline; gap: 12px;
    margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--line); color: var(--t-muted); font-size: 16px; font-weight: 600;
  }
  footer b { font: 900 28px/1 var(--display); color: var(--t-head); margin-right: 2px; }

  .canvas { height: min(640px, calc(100vh - 140px)); display: flex; flex-direction: column; gap: 12px; }
  .canvas :global(.wall) { flex: 1; }
  .key { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; justify-content: center; gap: 6px 18px; font-size: 13px; color: var(--t-muted); }
  .key li { display: flex; align-items: center; gap: 7px; opacity: 0; transition: opacity .5s; }
  .key li.on { opacity: 1; }
  .key i { width: 10px; height: 10px; flex: none; }
  .key .msg { border-radius: 3px; background: var(--h4); }
  .key .voice { border-radius: 50%; background: var(--cat-2); }
  .key .deleted { border-radius: 50%; box-shadow: inset 0 0 0 1.5px var(--t-head); }
  .key .game { border-radius: 3px; background: var(--cat-3); }

  @media (max-width: 900px) {
    .wrap { grid-template-columns: 1fr; align-items: start; gap: 20px; padding: 16px; }
    .canvas { order: -1; height: 300px; }
  }
</style>
