<script>
  import Emoji from "../components/Emoji.svelte";
  import { appInfo, soundUrl, stickerUrl } from "../lib/discord.js";
  import { fmt, monthYear, shortDate } from "../lib/stats.js";

  let { result } = $props();

  const facts = $derived(result.facts);
  const e = $derived(facts.emojis);
  const x = $derived(facts.expressions);
  const inMsgPct = $derived(e.total ? Math.round((e.in_messages / e.total) * 100) : 0);

  // Bot names for the commands and apps lists.
  let apps = $state({});
  const asked = new Set();
  $effect(() => {
    const ids = [...(x?.commands ?? []).slice(0, 15).map((c) => c.app), ...(x?.apps ?? []).slice(0, 12).map((a) => a[0])];
    for (const id of ids) {
      if (asked.has(id)) continue;
      asked.add(id);
      appInfo(id).then((a) => a && (apps = { ...apps, [id]: a }));
    }
  });

  // Discord's built-in sounds have small ids; the rest are server sounds.
  const DEFAULT_SOUNDS = { 1: "Quack", 2: "Airhorn", 3: "Cricket", 4: "Golf clap", 5: "Sad horn", 6: "Ba dum tss" };
  const soundName = (id, i) => DEFAULT_SOUNDS[id] ?? (/^\d{1,3}$/.test(id) ? `Built-in sound ${id}` : `Sound ${i + 1}`);
  // Discord only keeps the last couple of months of sound plays in the package.
  const soundSpan = $derived(x?.sounds_span ?? null);

  // One sound at a time.
  let audio = null;
  let playing = $state(null);
  function play(id) {
    audio?.pause();
    if (playing === id) { playing = null; return; }
    audio = new Audio(soundUrl(id));
    audio.volume = 0.6;
    audio.onended = () => (playing = null);
    audio.play().catch(() => (playing = null));
    playing = id;
  }

  // GIF videos load and play only while on screen (and never play with reduced motion on).
  function lazy(node, src) {
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const io = new IntersectionObserver(([en]) => {
      if (en.isIntersecting) {
        if (!node.src) node.src = src;
        if (!still) node.play().catch(() => {});
      } else node.pause();
    }, { rootMargin: "300px" });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  // Favourite GIFs point at Discord's media proxy, whose links can expire. The original address is
  // inside the proxy link, so try that next, then hide the tile.
  function gifError(ev) {
    const el = ev.currentTarget;
    const m = /\/external\/[^/]+\/(https?)\/(.+)$/.exec(el.src || el.currentSrc || "");
    if (m && !el.dataset.tried) { el.dataset.tried = "1"; el.src = `${m[1]}://${m[2]}`; }
    else el.closest("a").style.display = "none";
  }

  // Stickers can be PNG, APNG or GIF; try PNG, then GIF, then give up.
  function stickerError(ev, id) {
    const img = ev.currentTarget;
    if (!img.dataset.tried) { img.dataset.tried = "1"; img.src = stickerUrl(id, "gif"); }
    else img.replaceWith(Object.assign(document.createElement("span"), { className: "noimg", textContent: "Sticker" }));
  }
</script>

<div class="page">
  <section class="strip">
    <dl>
      <div><dt>Emoji used</dt><dd>{fmt(e.total)}</dd></div>
      <div><dt>In messages</dt><dd>{fmt(e.in_messages)}<small>{inMsgPct}%</small></dd></div>
      <div><dt>Reactions</dt><dd>{fmt(e.reactions)}</dd></div>
      {#if x}
        <div><dt>Favourite GIFs</dt><dd>{fmt(x.favorite_gifs.length)}</dd></div>
        <div><dt>Sticker uses</dt><dd>{fmt(x.stickers.reduce((a, s) => a + s[1], 0))}</dd></div>
      {/if}
    </dl>
  </section>

  <div class="grid12">
    <section class="card s12">
      <h2>Top emoji <small>(in messages and reactions)</small></h2>
      <ol class="emoji">
        {#each e.top as m, i (m.id ?? m.text)}
          <li class:big={i < 3} data-tip="{fmt(m.count)} times" data-sub={m.name ? `:${m.name}:` : ""}>
            <Emoji text={m.text} id={m.id} name={m.name} animated={m.animated} size={i < 3 ? 72 : 36} />
            <small class="num">{fmt(m.count)}</small>
          </li>
        {/each}
      </ol>
    </section>

    {#if x?.stickers.length}
      <section class="card s12">
        <h2>Top stickers</h2>
        <ol class="stickers">
          {#each x.stickers.slice(0, 12) as [id, n] (id)}
            <li data-tip="{fmt(n)} times">
              <img src={stickerUrl(id)} alt="Sticker" loading="lazy" onerror={(ev) => stickerError(ev, id)} />
              <small class="num">{fmt(n)}</small>
            </li>
          {/each}
        </ol>
      </section>
    {/if}

    {#if x?.favorite_gifs.length}
      <section class="card s12">
        <h2>Favourite GIFs</h2>
        <div class="gifs">
          {#each x.favorite_gifs as g (g.url)}
            <a href={g.url} target="_blank" rel="noreferrer noopener" style:aspect-ratio="{g.width || 1} / {g.height || 1}">
              {#if g.video}
                <video use:lazy={g.src} muted loop playsinline preload="metadata" aria-label="GIF" onerror={gifError}></video>
              {:else}
                <img src={g.src} alt="GIF" loading="lazy" onerror={gifError} />
              {/if}
            </a>
          {/each}
        </div>
      </section>
    {/if}

    {#if x?.sounds.length}
      <section class="card s6">
        <h2>Soundboard <small>{soundSpan ? `(Discord only kept ${monthYear(soundSpan[0])} – ${monthYear(soundSpan[1])})` : ""}</small></h2>
        <ol class="sounds">
          {#each x.sounds.slice(0, 12) as [id, n], i (id)}
            <li>
              <button class:on={playing === id} onclick={() => play(id)} aria-label="{playing === id ? 'Stop' : 'Play'} {soundName(id, i)}">
                {#if playing === id}
                  <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2" /></svg>
                {:else}
                  <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                {/if}
              </button>
              <span>{soundName(id, i)}{#if x.favorite_sounds.includes(id)} <b class="fav" title="Favourite">★</b>{/if}</span>
              <small class="num">{fmt(n)}×</small>
            </li>
          {/each}
        </ol>
        {#if x.favorite_sounds.length}
          <h3>Favourites</h3>
          <div class="favs">
            {#each x.favorite_sounds as id, i (id)}
              <button class:on={playing === id} onclick={() => play(id)} aria-label="{playing === id ? 'Stop' : 'Play'} favourite {i + 1}" data-tip={DEFAULT_SOUNDS[id] ?? "Favourite sound"}>
                {#if playing === id}
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2" /></svg>
                {:else}
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    {#if x?.commands.length}
      <section class="card s6">
        <h2>Slash commands</h2>
        <ol class="cmds">
          {#each x.commands.slice(0, 12) as c (c.app + c.name)}
            {@const a = apps[c.app]}
            <li>
              {#if a?.icon}<img src={a.icon} alt="" width="28" height="28" />{:else}<span class="ph"></span>{/if}
              <div><strong>/{c.name}</strong><small>{a?.name ?? "App"}</small></div>
              <span class="num">{fmt(c.uses)}×</span>
            </li>
          {/each}
        </ol>
      </section>
    {/if}
  </div>
</div>

<style>
  .emoji { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(72px, 1fr)); gap: 8px; }
  .emoji li { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; padding: 12px 4px; border-radius: 10px; background: var(--sunk); }
  .emoji li.big { grid-column: span 2; grid-row: span 2; }

  .emoji small { color: var(--t-muted); font-size: 12px; }
  .emoji li.big small { font-size: 14px; color: var(--t-head); font-weight: 700; }

  .stickers { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 10px; }
  .stickers li { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 12px; border-radius: 10px; background: var(--sunk); }
  .stickers img { width: 96px; height: 96px; object-fit: contain; }
  :global(.stickers .noimg) { width: 96px; height: 96px; display: grid; place-items: center; color: var(--t-faint); font-size: 12px; }
  .stickers small { color: var(--t-muted); font-size: 12px; }

  .gifs { columns: 5 180px; column-gap: 8px; }
  .gifs a { display: block; margin-bottom: 8px; border-radius: 8px; overflow: hidden; background: var(--sunk); break-inside: avoid; }
  .gifs img, .gifs video { display: block; width: 100%; height: 100%; object-fit: cover; }

  h3 { margin: 18px 0 8px; font-size: 13px; font-weight: 600; color: var(--t-muted); }
  .fav { color: var(--acc-text); font-weight: 400; }
  .favs { display: flex; flex-wrap: wrap; gap: 6px; }
  .favs button {
    width: 32px; height: 32px; border-radius: 50%; border: 0; display: grid; place-items: center; cursor: pointer;
    background: var(--raised); color: var(--t-head);
  }
  .favs button:hover, .favs button.on { background: var(--acc-strong); color: var(--on-acc); }
  .sounds, .cmds { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  .sounds li, .cmds li { display: grid; grid-template-columns: 32px 1fr auto; align-items: center; gap: 12px; }
  .sounds button {
    width: 32px; height: 32px; border-radius: 50%; border: 0; display: grid; place-items: center; cursor: pointer;
    background: var(--raised); color: var(--t-head);
  }
  .sounds button:hover, .sounds button.on { background: var(--acc-strong); color: var(--on-acc); }
  .sounds span { color: var(--t-head); font-weight: 600; }
  .sounds small, .cmds .num { color: var(--t-muted); font-size: 13px; }
  .cmds img, .cmds .ph { width: 28px; height: 28px; border-radius: 8px; display: block; }
  .cmds .ph { background: var(--raised); }
  .cmds div { min-width: 0; }
  .cmds strong { display: block; color: var(--t-head); font-weight: 600; }
  .cmds small { color: var(--t-muted); font-size: 12px; }
</style>
