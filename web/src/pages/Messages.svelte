<script>
  import Columns from "../components/Columns.svelte";
  import DotTimeline from "../components/DotTimeline.svelte";
  import Donut from "../components/Donut.svelte";
  import Emoji from "../components/Emoji.svelte";
  import Message from "../components/Message.svelte";
  import { eventCounts } from "../lib/eventStats.js";
  import { channelName, fmt, messageStats, monthYear, shortDate, tnsStats } from "../lib/stats.js";

  let { result, view, range, games } = $props();

  const facts = $derived(result.facts);
  const profile = $derived(facts.profile);
  const s = $derived(messageStats(view, facts, range));
  const LENGTHS = ["None", "1", "2", "3–5", "6–10", "11–20", "21–50", "51+"];
  const inRange = (ms) => range === "all" || new Date(ms).getUTCFullYear() === range;

  const where = $derived([
    { label: "Servers", value: s.where.guild, colour: "var(--cat-1)" },
    { label: "DMs", value: s.where.dm, colour: "var(--cat-2)" },
    { label: "Group chats", value: s.where.group, colour: "var(--cat-3)" },
    ...(s.where.other ? [{ label: "Other", value: s.where.other, colour: "var(--cat-4)" }] : []),
  ]);

  // Filler words ("the", "and", "de", "het"…) are left out unless you switch them on.
  let withFiller = $state(false);
  const words = $derived(withFiller ? [...s.words_top, ...s.fillers].sort((a, b) => b.n - a.n).slice(0, 30) : s.words_top.slice(0, 30));
  const topWord = $derived(words[0]?.n ?? 1);
  const topChannel = $derived(s.channels[0]?.n ?? 1);
  const topDomain = $derived(s.domains[0]?.n ?? 1);

  // Where a message was sent, in words.
  const servers = $derived(new Map(facts.servers.map((x) => [x.id, x.name])));
  const channels = $derived(new Map(facts.channels.map((c) => [c.id, c])));
  function place(channelId) {
    const c = channels.get(channelId);
    if (!c) return "";
    if (c.kind === "dm") return `DM with ${view.people.get(c.person)?.display_name ?? "someone"}`;
    if (c.kind === "group") return c.name && c.name !== "None" ? c.name : "a group chat";
    if (c.kind === "guild") { const n = channelName(c, servers.get(c.guild)); return `#${n.channel} · ${n.server}`; }
    return "";
  }

  const author = $derived({
    id: profile?.id ?? "",
    avatar: profile?.avatar,
    src: result.avatar ? URL.createObjectURL(result.avatar) : null,
    name: profile?.display_name ?? "You",
  });

  // The longest messages, with the same text sent again within minutes folded into one entry.
  const longest = $derived.by(() => {
    const out = [];
    for (const q of facts.text.longest.filter((x) => inRange(x.ms))) {
      const same = out.find((o) => o.channel === q.channel && o.chars === q.chars && Math.abs(o.ms - q.ms) < 600_000 && o.text === q.text);
      if (same) same.times++;
      else out.push({ ...q, times: 1, key: `${q.ms}-${q.channel}` });
    }
    return out;
  });
  const walls = $derived(facts.text.walls.filter((w) => inRange(w.start_ms)).map((w) => ({ ...w, key: `${w.start_ms}-${w.channel}` })));
  let openLong = $state(null);
  let openWall = $state(null);
  const span = $derived([profile?.created_ms ?? Math.min(...facts.text.longest.map((q) => q.ms)), result.sources?.packageDate || Date.now()]);
  const minutes = (ms) => Math.max(1, Math.round(ms / 60_000));
  // Edits: per month from the trust & safety log, so a year can be picked; for all time, the
  // highest count any log kept (the analytics log often goes back further), like Statistics.
  const tns = $derived(tnsStats(facts, range));
  const edits = $derived.by(() => {
    const logged = range === "all" ? eventCounts(result, games).get("message_edited") ?? 0 : 0;
    if (logged > (tns?.edits ?? 0)) return { n: logged };
    return tns ? { n: tns.edits, since: tns.since } : null;
  });
</script>

<div class="wrap">
  <section class="card stats">
    <dl>
      <div><dt>Messages</dt><dd class="num">{fmt(s.messages)}</dd></div>
      <div><dt>Words</dt><dd class="num">{fmt(s.words)}</dd></div>
      <div><dt>Words per message</dt><dd class="num">{(s.words / Math.max(s.messages, 1)).toFixed(1)}</dd></div>
      <div><dt>With attachments</dt><dd class="num">{fmt(s.attachments)}</dd></div>
      <div><dt>With links</dt><dd class="num">{fmt(s.links)}</dd></div>
      {#if edits}<div data-tip={edits.since ? `Since ${monthYear(edits.since)}` : null}><dt>Edited</dt><dd class="num">{fmt(edits.n)}</dd></div>{/if}
    </dl>
  </section>

  <div class="grid">
    <section class="card span5">
      <h2>Where you talk</h2>
      <Donut segments={where} centre={fmt(s.messages)} centreSub="messages" label="Messages by where you sent them" />
    </section>

    <section class="card span7">
      <h2>Message length <small>(words per message)</small></h2>
      <Columns
        bars={s.lengths.map((value, i) => ({ label: LENGTHS[i], value, tip: i === 0 ? "No text: attachment or sticker" : `${LENGTHS[i]} word${i === 1 ? "" : "s"}` }))}
        label="Messages by length in words"
      />
    </section>

    <section class="card span7">
      <div class="whead">
        <h2>Top words</h2>
        <label class="switch">
          <input type="checkbox" role="switch" bind:checked={withFiller} />
          <span aria-hidden="true"></span>
          Filler words
        </label>
      </div>
      <ol class="words">
        {#each words as w, i (w.text)}
          <li>
            <span class="rk">{i + 1}</span>
            <span class="w">{w.text}</span>
            <span class="bar"><i style:width="{(w.n / topWord) * 100}%"></i></span>
            <span class="n num">{fmt(w.n)}</span>
          </li>
        {/each}
      </ol>
    </section>

    <section class="card span5">
      <h2>Top channels</h2>
      <ol class="rank">
        {#each s.channels as c (c.id)}
          <li>
            <div><strong>#{c.channel}</strong><small>{c.server}</small></div>
            <span class="n num">{fmt(c.n)}</span>
            <span class="bar"><i style:width="{(c.n / topChannel) * 100}%"></i></span>
          </li>
        {:else}
          <li class="empty">No server messages in this range.</li>
        {/each}
      </ol>
    </section>

    <section class="card span5">
      <h2>Links</h2>
      <ol class="rank">
        {#each s.domains as d (d.text)}
          <li>
            <div><strong>{d.text}</strong></div>
            <span class="n num">{fmt(d.n)}</span>
            <span class="bar"><i style:width="{(d.n / topDomain) * 100}%"></i></span>
          </li>
        {:else}
          <li class="empty">No links in this range.</li>
        {/each}
      </ol>
    </section>

    <section class="card span7">
      <h2>Top emoji <small>(in messages and reactions{range === "all" ? "" : ", all time"})</small></h2>
      <ol class="emoji">
        {#each facts.emojis.top.slice(0, 15) as e (e.id ?? e.text)}
          <li data-tip="{fmt(e.count)} times" data-sub={e.name ? `:${e.name}:` : ""}>
            <Emoji text={e.text} id={e.id} name={e.name} animated={e.animated} size={32} />
            <small class="num">{fmt(e.count)}</small>
          </li>
        {/each}
      </ol>
    </section>

    {#if facts.text.first}
      <section class="card span12">
        <h2>First message <small>(the oldest one still in your package)</small></h2>
        <Message {author} parts={[{ ms: facts.text.first.ms, text: facts.text.first.text }]} place={place(facts.text.first.channel)} tz={view.tz} people={view.people} />
      </section>
    {/if}

    {#if longest.length}
      <section class="card span12">
        <h2>Longest messages</h2>
        <DotTimeline
          points={longest.map((q) => ({ key: q.key, ms: q.ms, value: q.chars, tip: `${fmt(q.chars)} characters`, sub: `${shortDate(q.ms)} · ${place(q.channel)}` }))}
          from={span[0]}
          to={span[1]}
          format={fmt}
          active={openLong}
          onpick={(k) => (openLong = openLong === k ? null : k)}
        />
        <ol class="board">
          {#each longest as q, i (q.key)}
            <li class:open={openLong === q.key}>
              <button class="row" aria-expanded={openLong === q.key} onclick={() => (openLong = openLong === q.key ? null : q.key)}>
                <span class="rk">{i + 1}</span>
                <span class="len num">{fmt(q.chars)} <small>characters</small></span>
                <span class="where">{place(q.channel)}{q.times > 1 ? ` · sent ${q.times} times` : ""}</span>
                <span class="date">{shortDate(q.ms)}</span>
              </button>
              {#if openLong === q.key}
                <div class="full">
                  <Message {author} parts={[{ ms: q.ms, text: q.text }]} place={place(q.channel)} tz={view.tz} people={view.people} cut={q.chars > q.text.length} />
                </div>
              {/if}
            </li>
          {/each}
        </ol>
      </section>
    {/if}

    {#if walls.length}
      <section class="card span12">
        <h2>Walls of text <small>(long messages sent one after another)</small></h2>
        <ol class="board">
          {#each walls as w, i (w.key)}
            <li class:open={openWall === w.key}>
              <button class="row" aria-expanded={openWall === w.key} onclick={() => (openWall = openWall === w.key ? null : w.key)}>
                <span class="rk">{i + 1}</span>
                <span class="len num">{fmt(w.count)} <small>messages</small> · {fmt(w.chars)} <small>characters</small></span>
                <span class="where">{place(w.channel)} · {minutes(w.end_ms - w.start_ms)} min</span>
                <span class="date">{shortDate(w.start_ms)}</span>
              </button>
              {#if openWall === w.key}
                <div class="full">
                  <Message {author} parts={w.parts.map(([ms, text]) => ({ ms, text }))} place={place(w.channel)} tz={view.tz} people={view.people} cut={w.count > w.parts.length} />
                </div>
              {/if}
            </li>
          {/each}
        </ol>
      </section>
    {/if}
  </div>
</div>

<style>
  .wrap { max-width: 1320px; margin: 0 auto; padding: 24px; }
  .grid { display: grid; grid-template-columns: repeat(12, 1fr); gap: 16px; margin-top: 16px; }
  .span5 { grid-column: span 5; }
  .span7 { grid-column: span 7; }
  .span12 { grid-column: span 12; }

  .stats { padding: 0; }
  .stats dl { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); margin: 0; }
  .stats div { padding: 18px 24px; }
  .stats div + div { border-left: 1px solid var(--line); }
  .stats dt { font-size: 14px; color: var(--t-muted); font-weight: 600; }
  .stats dd { margin: 4px 0 0; font: 900 26px/1.1 var(--display); color: var(--t-head); }

  .rk { color: var(--t-faint); font-weight: 700; font-size: 13px; text-align: right; }
  .n { color: var(--t-muted); font-size: 13px; text-align: right; }
  .bar { display: block; height: 6px; border-radius: 3px; background: var(--sunk); }
  .bar i { display: block; height: 100%; border-radius: 3px; background: var(--acc); }

  .whead { display: flex; justify-content: space-between; align-items: start; gap: 16px; }
  .switch { display: flex; align-items: center; gap: 8px; font-size: 14px; font-weight: 600; color: var(--t-body); cursor: pointer; white-space: nowrap; }
  .switch input { position: absolute; opacity: 0; width: 1px; height: 1px; }
  .switch span { width: 40px; height: 24px; border-radius: 12px; background: var(--t-faint); position: relative; transition: background .15s; flex: none; }
  .switch span::after { content: ""; position: absolute; top: 3px; left: 3px; width: 18px; height: 18px; border-radius: 50%; background: #fff; transition: transform .15s; }
  .switch input:checked + span { background: var(--acc-strong); }
  .switch input:checked + span::after { transform: translateX(16px); }
  .switch input:focus-visible + span { outline: 2px solid var(--acc-text); outline-offset: 2px; }
  .words { list-style: none; margin: 0; padding: 0; columns: 2; column-gap: 28px; }
  .words li { display: grid; grid-template-columns: 22px minmax(0, 1fr) 70px 52px; align-items: center; gap: 8px; padding: 4px 0; break-inside: avoid; }
  .words .w { color: var(--t-head); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .rank { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .rank li { display: grid; grid-template-columns: 1fr auto; gap: 4px 12px; align-items: end; }
  .rank li .bar { grid-column: 1 / -1; }
  .rank strong { color: var(--t-head); font-weight: 600; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rank small { color: var(--t-muted); font-size: 12px; }
  .rank div { min-width: 0; }
  .empty { color: var(--t-muted); display: block !important; }

  .emoji { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(64px, 1fr)); gap: 8px; }
  .emoji li { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 10px 4px; border-radius: 8px; background: var(--sunk); }
  .emoji li:hover { background: var(--raised); }
  .emoji small { color: var(--t-muted); font-size: 12px; }

  .board { list-style: none; margin: 12px 0 0; padding: 0; display: grid; gap: 2px; }
  .board li { border-radius: 8px; }
  .board li.open { background: var(--sunk); }
  .row {
    width: 100%; display: grid; grid-template-columns: 26px 240px minmax(0, 1fr) auto; align-items: baseline; gap: 14px;
    padding: 10px 10px; border: 0; border-radius: 8px; background: none; color: inherit; text-align: left; cursor: pointer;
  }
  .row:hover { background: var(--hover); }
  .open .row:hover { background: none; }
  .len { color: var(--t-head); font-weight: 700; }
  .len small { color: var(--t-muted); font-weight: 500; font-size: 12px; }
  .where { color: var(--t-body); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .date { color: var(--t-muted); font-size: 13px; white-space: nowrap; }
  .full { padding: 6px 16px 16px 50px; max-height: 520px; overflow-y: auto; }

  @media (max-width: 900px) {
    .span5, .span7 { grid-column: span 12; }
    .stats div:nth-child(n) { border-left: 0; border-top: 1px solid var(--line); }
    .stats div:first-child { border-top: 0; }
  }
  @media (max-width: 620px) {
    .wrap { padding: 16px; }
    .words { columns: 1; }
    .card { padding: 16px; }
    .row { grid-template-columns: 22px 1fr; }
    .where, .date { grid-column: 2; }
    .full { padding: 6px 8px 12px; }
  }
</style>
