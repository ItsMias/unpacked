<script>
  // The home page's tour of the pages: one tile per page with a taste of it from the demo
  // account. Each opens that page of the demo.
  import Avatar from "./Avatar.svelte";
  import Emoji from "./Emoji.svelte";
  import Icon from "./Icon.svelte";
  import ServerIcon from "./ServerIcon.svelte";
  import { appInfo } from "../lib/discord.js";
  import { deviceName, price } from "../lib/names.js";
  import { fmt, fmtCompact, shortMonth } from "../lib/stats.js";

  let { sample, ondemo } = $props();

  const TILES = [
    { id: "messages", label: "Messages", icon: "chat", wide: true },
    { id: "people", label: "People", icon: "people" },
    { id: "servers", label: "Servers", icon: "servers" },
    { id: "voice", label: "Voice", icon: "voice" },
    { id: "games", label: "Games", icon: "games", wide: true },
    { id: "timeline", label: "Timeline", icon: "timeline" },
    { id: "emoji", label: "Emoji & GIFs", icon: "emoji" },
    { id: "devices", label: "Devices", icon: "devices" },
    { id: "purchases", label: "Purchases", icon: "gift" },
    { id: "privacy", label: "What Discord knows", icon: "eye" },
  ];

  function open(e, id) {
    // A plain click opens the demo here; middle and modified clicks follow the link to a new tab.
    if (e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    ondemo(id);
  }

  // Messages per month as one smooth line, for the Messages tile.
  const spark = $derived.by(() => {
    if (!sample) return null;
    const pts = sample.months;
    const max = Math.max(...pts.map((p) => p.alive), 1);
    const W = 600, H = 96;
    const xy = pts.map((p, i) => [(i / (pts.length - 1)) * W, H - 4 - (p.alive / max) * (H - 8)]);
    const line = xy.map(([x, y], i) => `${i ? "L" : "M"}${x.toFixed(1)} ${y.toFixed(1)}`).join("");
    const years = [];
    pts.forEach((p, i) => new Date(p.t).getUTCMonth() === 0 && years.push({ x: (i / (pts.length - 1)) * 100, y: new Date(p.t).getUTCFullYear() }));
    return { line, area: `${line}L${W} ${H}L0 ${H}Z`, years };
  });

  // Game art, from Discord's public app info (and Steam covers where there are any).
  let apps = $state({});
  let broken = $state({});
  $effect(() => {
    for (const g of sample?.games ?? []) appInfo(g.id).then((a) => a && (apps = { ...apps, [g.id]: a }));
  });

  const DEVICE_ICON = { android: "phone", ios: "phone", desktop: "desktop", web: "web", other: "devices" };
  const deviceLabel = (c) => (c.device ? deviceName(c.device) : c.platform === "desktop" ? `Discord on ${c.os || "desktop"}` : `${c.browser}${c.os ? ` on ${c.os}` : ""}`);
  const spent = $derived(sample?.money.spent[0]);
  const maxPerson = $derived(sample?.people[0]?.messages ?? 1);
  const maxServer = $derived(sample?.servers[0]?.messages ?? 1);
  const maxVoice = $derived(Math.max(1, ...(sample?.voiceYears ?? []).map((v) => v[1])));
  const label = (s) => s.replace(/\b\w/g, (c) => c.toUpperCase());
</script>

<div class="bento">
  {#each TILES as t (t.id)}
    <a class="tile" class:wide={t.wide} href="?demo#/{t.id}" aria-label="{t.label}, in the demo" onclick={(e) => open(e, t.id)}>
      <header>
        <span class="ic"><Icon name={t.icon} size={18} /></span>
        <h3>{t.label}</h3>
        <span class="go"><Icon name="chevron" size={18} /></span>
      </header>

      {#if sample}
        <div class="viz">
          {#if t.id === "messages"}
            <dl class="nums">
              <div><dt>Sent</dt><dd class="num">{fmt(sample.o.sent)}</dd></div>
              <div><dt>Still there</dt><dd class="num">{Math.round((sample.o.alive / sample.o.sent) * 100)}%</dd></div>
              <div><dt>Words</dt><dd class="num">{fmtCompact(sample.messages.words)}</dd></div>
            </dl>
            <div class="spark">
              <svg viewBox="0 0 600 96" preserveAspectRatio="none">
                <defs>
                  <linearGradient id="spark-fill" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0" stop-color="var(--acc)" stop-opacity=".45" />
                    <stop offset="1" stop-color="var(--acc)" stop-opacity="0" />
                  </linearGradient>
                </defs>
                <path d={spark.area} fill="url(#spark-fill)" />
                <path d={spark.line} fill="none" stroke="var(--acc-text)" stroke-width="2" vector-effect="non-scaling-stroke" />
              </svg>
              <div class="years">{#each spark.years as y (y.y)}<span style:left="{y.x}%">{y.y}</span>{/each}</div>
            </div>
            <div class="words">{#each sample.words as w (w.text)}<span>{w.text}</span>{/each}</div>

          {:else if t.id === "people"}
            <ol class="rank">
              {#each sample.people as p (p.id)}
                <li>
                  <Avatar id={p.id} name={p.display_name} size={30} />
                  <span class="who"><b>{p.display_name}</b><span class="meter"><i style:width="{(p.messages / maxPerson) * 100}%"></i></span></span>
                  <span class="n num">{fmtCompact(p.messages)}</span>
                </li>
              {/each}
            </ol>

          {:else if t.id === "servers"}
            <ol class="rank">
              {#each sample.servers as s (s.id)}
                <li>
                  <ServerIcon name={s.name} size={30} />
                  <span class="who"><b>{s.name}</b><span class="meter"><i style:width="{(s.messages / maxServer) * 100}%"></i></span></span>
                  <span class="n num">{fmtCompact(s.messages)}</span>
                </li>
              {/each}
            </ol>

          {:else if t.id === "voice"}
            <p class="big num">{fmt(sample.voice.hours)}<small>hours</small></p>
            <div class="cols">
              {#each sample.voiceYears as [y, h] (y)}
                <span><i style:height="{(h / maxVoice) * 100}%"></i><small>{String(y).slice(2)}</small></span>
              {/each}
            </div>

          {:else if t.id === "games"}
            <div class="covers">
              {#each sample.games as g (g.id)}
                {@const a = apps[g.id]}
                <figure>
                  <div class="art">
                    {#if a?.cover && !broken[g.id]}
                      <img class="cover" src={a.cover} alt="" loading="lazy" onerror={() => (broken = { ...broken, [g.id]: true })} />
                    {:else if a?.icon}
                      <img class="blur" src={a.icon} alt="" /><img class="icon" src={a.icon} alt="" />
                    {/if}
                  </div>
                  <figcaption><b>{g.name}</b><small class="num">{fmt(g.rangeHours)} h</small></figcaption>
                </figure>
              {/each}
            </div>

          {:else if t.id === "timeline"}
            <ol class="line">
              {#each sample.timeline as it (it.ms)}
                <li><time>{shortMonth(it.ms)}</time><b>{it.title}</b></li>
              {/each}
            </ol>

          {:else if t.id === "emoji"}
            <ol class="emoji">
              {#each sample.emoji as e (e.text)}
                <li><Emoji text={e.text} size={30} /><small class="num">{fmtCompact(e.count)}</small></li>
              {/each}
            </ol>

          {:else if t.id === "devices"}
            <ol class="rank devices">
              {#each sample.devices as c (c.platform + c.device + c.browser)}
                <li>
                  <span class="dic"><Icon name={DEVICE_ICON[c.platform] ?? "devices"} size={16} /></span>
                  <span class="who"><b>{deviceLabel(c)}</b></span>
                  <span class="n num">{fmtCompact(c.count)}</span>
                </li>
              {/each}
            </ol>

          {:else if t.id === "purchases"}
            {#if spent}<p class="big num">{price(spent[1], spent[0])}<small>spent</small></p>{/if}
            <div class="pills">
              <span><Icon name="gift" size={14} />{sample.money.made.length} gifts given</span>
              <span><Icon name="star" size={14} />{sample.money.boosts} boosts</span>
            </div>

          {:else if t.id === "privacy"}
            <dl class="secret">
              <div><dt>Age group</dt><dd>{sample.ads.age_group}</dd></div>
              <div><dt>Region</dt><dd>{sample.ads.reg_region}</dd></div>
              <div><dt>Interests</dt><dd>{sample.ads.theme_names_l90.map(label).join(", ")}</dd></div>
            </dl>
          {/if}
        </div>
      {/if}
    </a>
  {/each}
  <a class="tile all" href="?demo#/overview" aria-label="Open the whole demo" onclick={(e) => open(e, "overview")}>
    &amp; way more<span class="arrow"><Icon name="chevron" size={26} stroke={2.6} /></span>
  </a>
</div>

<style>
  .bento { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 16px; }
  .tile {
    position: relative; display: flex; flex-direction: column; gap: 14px; min-width: 0; min-height: 210px;
    padding: 18px 20px 20px; border-radius: var(--radius); background: var(--card);
    color: var(--t-body); text-decoration: none;
    transition: background .15s, box-shadow .2s, translate .2s;
  }
  .tile.wide { grid-column: span 2; }
  .tile:hover, .tile:focus-visible { background: var(--hover); translate: 0 -2px; box-shadow: 0 10px 28px rgba(0, 0, 0, .3), 0 0 0 1px var(--acc-soft); }

  header { display: flex; align-items: center; gap: 10px; }
  .ic { width: 32px; height: 32px; border-radius: 10px; display: grid; place-items: center; background: var(--acc-soft); color: var(--acc-text); }
  h3 { margin: 0; font-size: 16px; font-weight: 700; color: var(--t-head); }
  .go { margin-left: auto; color: var(--t-muted); opacity: 0; translate: -4px 0; transition: opacity .15s, translate .15s; }
  .tile:hover .go, .tile:focus-visible .go { opacity: 1; translate: 0 0; color: var(--acc-text); }
  .viz { flex: 1; display: flex; flex-direction: column; gap: 14px; min-width: 0; }

  /* The last tile: the whole demo, glowing from behind its words. */
  .tile.all {
    grid-column: 1 / -1; min-height: 0; flex-direction: row; align-items: center; justify-content: center; gap: 6px;
    padding: 26px; font: 900 26px/1 var(--display); color: var(--t-head);
    background:
      radial-gradient(42% 160% at 50% 50%, color-mix(in srgb, var(--acc) 34%, transparent), transparent 72%),
      var(--card);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--acc) 30%, transparent);
  }
  .tile.all:hover, .tile.all:focus-visible {
    background:
      radial-gradient(48% 170% at 50% 50%, color-mix(in srgb, var(--acc) 46%, transparent), transparent 74%),
      var(--hover);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--acc) 55%, transparent), 0 10px 36px color-mix(in srgb, var(--acc) 22%, transparent);
  }
  .arrow { display: grid; color: var(--acc-text); transition: translate .2s; }
  .tile.all:hover .arrow, .tile.all:focus-visible .arrow { translate: 5px 0; }

  .nums { display: flex; gap: 28px; margin: 0; }
  .nums dt, .secret dt { font-size: 12px; font-weight: 600; color: var(--t-muted); }
  .nums dd { margin: 2px 0 0; font: 900 22px/1.1 var(--display); color: var(--t-head); }
  .spark { position: relative; padding-bottom: 16px; }
  .spark svg { display: block; width: 100%; height: 84px; }
  .years span { position: absolute; bottom: 0; translate: -50% 0; font-size: 11px; color: var(--t-faint); }
  .words { display: flex; flex-wrap: wrap; gap: 6px; }
  .words span { padding: 3px 9px; border-radius: 999px; background: var(--raised); font-size: 13px; font-weight: 600; color: var(--t-body); }

  .rank { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .rank li { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; }
  .who { min-width: 0; display: grid; gap: 5px; }
  .who b { font-size: 14px; font-weight: 600; color: var(--t-head); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .who .meter { height: 4px; }
  .n { font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .dic { width: 30px; height: 30px; border-radius: 9px; background: var(--raised); color: var(--t-body); display: grid; place-items: center; }

  .big { margin: 0; font: 900 32px/1 var(--display); color: var(--t-head); }
  .big small { font: 600 14px var(--ui); color: var(--t-muted); margin-left: 6px; }
  .cols { flex: 1; min-height: 70px; display: flex; align-items: stretch; gap: 6px; }
  .cols span { flex: 1; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: 4px; }
  .cols i { width: 100%; min-height: 3px; border-radius: 4px 4px 2px 2px; background: var(--acc); }
  .cols small { font-size: 11px; color: var(--t-faint); }

  .covers { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
  figure { margin: 0; min-width: 0; }
  .art { position: relative; aspect-ratio: 3 / 4; border-radius: 8px; overflow: hidden; background: var(--raised); }
  .art img { position: absolute; }
  .cover { inset: 0; width: 100%; height: 100%; object-fit: cover; }
  .blur { inset: -20%; width: 140%; height: 140%; object-fit: cover; filter: blur(22px) saturate(1.3); opacity: .6; }
  .icon { left: 50%; top: 50%; width: 44%; translate: -50% -50%; border-radius: 12px; box-shadow: 0 4px 14px rgba(0, 0, 0, .4); }
  figcaption { margin-top: 8px; display: grid; }
  figcaption b { font-size: 13px; font-weight: 600; color: var(--t-head); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  figcaption small { font-size: 12px; color: var(--t-muted); }

  .line { list-style: none; margin: 0; padding: 0 0 0 18px; display: grid; gap: 12px; border-left: 2px solid var(--line); margin-left: 5px; }
  .line li { position: relative; display: grid; }
  .line li::before { content: ""; position: absolute; left: -25px; top: 4px; width: 12px; height: 12px; border-radius: 50%; background: var(--acc); box-shadow: 0 0 0 3px var(--card); }
  .tile:hover .line li::before { box-shadow: 0 0 0 3px var(--hover); }
  .line time { font-size: 12px; color: var(--t-muted); }
  .line b { font-size: 14px; font-weight: 600; color: var(--t-head); }

  .emoji { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px 8px; }
  .emoji li { display: grid; justify-items: center; gap: 4px; }
  .emoji small { font-size: 12px; font-weight: 600; color: var(--t-muted); }

  .pills { display: flex; flex-wrap: wrap; gap: 6px; }
  .pills span { display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; border-radius: 999px; background: var(--raised); font-size: 13px; font-weight: 600; color: var(--t-body); }
  .pills :global(.icon) { color: var(--acc-text); }

  .secret { display: grid; gap: 10px; margin: 0; }
  .secret dd { margin: 2px 0 0; width: fit-content; font-size: 14px; font-weight: 600; color: var(--t-head); filter: blur(6px); transition: filter .25s; }
  .tile:hover .secret dd, .tile:focus-visible .secret dd { filter: none; }

  @media (max-width: 1100px) {
    .bento { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  @media (max-width: 620px) {
    .bento { grid-template-columns: minmax(0, 1fr); }
    .tile.wide { grid-column: auto; }
    .nums { gap: 18px; }
    .covers { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
</style>
