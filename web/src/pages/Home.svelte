<script>
  import { onMount } from "svelte";
  import HeroWall from "../components/HeroWall.svelte";
  import Icon from "../components/Icon.svelte";
  import Preview from "../components/Preview.svelte";
  import Showcase from "../components/Showcase.svelte";
  import { summarize } from "../lib/showcase.js";
  import { REPO } from "../lib/site.js";

  /** `ondemo(page?)` opens the demo account, on `page` if given. */
  let { phase, error, onfile, ondemo } = $props();

  let dragging = $state(false);
  // The demo account's numbers for the previews, made once the page has painted.
  let sample = $state.raw(null);

  function pick(files) {
    const f = files?.[0];
    if (f) onfile(f);
  }

  onMount(() => {
    let live = true;
    const later = window.requestIdleCallback ?? ((f) => setTimeout(f, 30));
    later(async () => {
      const { demoResult } = await import("../lib/demo.js");
      if (live) sample = summarize(demoResult());
    });
    return () => (live = false);
  });
</script>

<main class="home">
  <section class="hero">
    <HeroWall />
    <div class="in">
      <div class="pitch">
        <h1>Unpacked</h1>
        <p class="lede">Stats on your messages, calls, servers and games, from your Discord data package.</p>

        <label
          class="drop"
          class:dragging
          ondragover={(e) => { e.preventDefault(); dragging = true; }}
          ondragleave={() => (dragging = false)}
          ondrop={(e) => { e.preventDefault(); dragging = false; pick(e.dataTransfer.files); }}
        >
          <input type="file" accept=".zip,application/zip" class="sr-only" onchange={(e) => pick(e.currentTarget.files)} />
          <span class="up"><Icon name="upload" size={24} /></span>
          <span class="what"><strong>Drop your package here</strong><small>The .zip file from Discord</small></span>
          <span class="choose">Choose file</span>
        </label>
        {#if phase === "error"}
          <p class="error" role="alert">{error}</p>
        {/if}

        <div class="row">
          <button class="demo" onclick={() => ondemo()}><Icon name="play" size={16} />Try the demo</button>
          <span class="local"><Icon name="lock" size={16} />Read in your browser. Nothing is uploaded.</span>
        </div>
      </div>

      <Preview {sample} />
    </div>
  </section>

  <section class="band">
    <h2>Preview</h2>
    <Showcase {sample} {ondemo} />
  </section>

  <section class="band">
    <h2>Getting your package</h2>
    <ol class="steps">
      <li>
        <span class="step"><Icon name="cog" size={22} /></span>
        <p>In Discord, open <span class="path"><span class="key">User Settings</span><Icon name="chevron" size={16} /><span class="key">Data &amp; Privacy</span></span></p>
      </li>
      <li>
        <span class="step"><Icon name="download" size={22} /></span>
        <p>Choose <span class="key">Request all of my data</span> and select everything.</p>
      </li>
      <li>
        <span class="step"><Icon name="mail" size={22} /></span>
        <p>Discord emails you a download link, usually within a few days.</p>
      </li>
    </ol>
  </section>

  <section class="band faq">
    <h2>Questions</h2>
    <details>
      <summary>Is my package uploaded anywhere?<Icon name="down" size={18} /></summary>
      <p>No. The zip is read inside this browser tab and nothing from it leaves your device. It's gone when you close the tab. Avatars, server icons and game art load from Discord and Steam.</p>
    </details>
    <details>
      <summary>How do I know this is safe?<Icon name="down" size={18} /></summary>
      <p>
        The whole site is open source on <a href={REPO} target="_blank" rel="noopener">GitHub</a>, so anyone can read exactly what it
        does with your package. Your zip is read inside your browser and never uploaded. You can check this yourself: open your
        browser's developer tools, go to the Network tab, and drop your package in. The only requests are for pictures and names
        from Discord and Steam.
      </p>
    </details>
    <details>
      <summary>What if "Use data to improve Discord" was off?<Icon name="down" size={18} /></summary>
      <p>Everything works. Without that setting the package has no analytics folder, so the Games page only shows games you played while in a voice channel.</p>
    </details>
    <details>
      <summary>How long does it take?<Icon name="down" size={18} /></summary>
      <p>Usually under a minute, even for packages of several gigabytes. Game history keeps loading in the background while you look around.</p>
    </details>
  </section>

  <footer>
    <a class="credit" href="https://itsmias.xyz" target="_blank" rel="noopener">Made by ItsMias</a><span class="sep" aria-hidden="true">-</span>Not affiliated with or endorsed by Discord.
  </footer>
</main>

<style>
  .hero { position: relative; overflow: hidden; }
  /* The wall fades out behind the text and towards the bottom. */
  .hero :global(.wall) {
    opacity: .5;
    mask-image: linear-gradient(to bottom, #000 55%, transparent 98%), linear-gradient(to right, rgba(0, 0, 0, .15) 15%, #000 62%);
    mask-composite: intersect;
  }
  .in {
    position: relative; max-width: 1320px; margin: 0 auto; padding: 88px 24px 80px;
    display: grid; grid-template-columns: minmax(0, 560px) minmax(0, 1fr); gap: 56px; align-items: center;
  }
  .pitch > * { animation: rise .7s cubic-bezier(.2, .8, .2, 1) both; }
  .pitch > :nth-child(2) { animation-delay: 60ms; }
  .pitch > :nth-child(n + 3) { animation-delay: 120ms; }
  @keyframes rise { from { opacity: 0; translate: 0 14px; } }

  h1 { margin: 0; font: 900 clamp(52px, 7.2vw, 92px)/1 var(--display); letter-spacing: -.02em; color: var(--t-head); }
  .lede { margin: 16px 0 32px; font-size: 19px; line-height: 1.45; color: var(--t-body); max-width: 30em; }

  .drop {
    display: flex; align-items: center; gap: 16px; padding: 18px 18px 18px 20px; cursor: pointer;
    border-radius: var(--radius); border: 2px dashed var(--line); background: color-mix(in srgb, var(--card) 88%, transparent);
    backdrop-filter: blur(6px); transition: border-color .15s, background .15s;
  }
  .drop:hover, .drop.dragging, .drop:focus-within { border-color: var(--acc); background: var(--hover); }
  .up { width: 48px; height: 48px; border-radius: 14px; display: grid; place-items: center; background: var(--acc-soft); color: var(--acc-text); flex: none; }
  .what { flex: 1; min-width: 0; display: grid; }
  .what strong { font-size: 17px; color: var(--t-head); }
  .what small { font-size: 14px; color: var(--t-muted); }
  .choose { padding: 10px 16px; border-radius: 8px; background: var(--acc-strong); color: var(--on-acc); font-weight: 600; font-size: 15px; white-space: nowrap; }
  .drop:hover .choose { filter: brightness(1.1); }

  .error { margin: 12px 0 0; padding: 12px 14px; border-radius: 8px; background: rgba(242, 63, 67, .12); color: #ffb3b5; font-size: 14px; }

  .row { display: flex; align-items: center; flex-wrap: wrap; gap: 12px 20px; margin-top: 18px; }
  .demo {
    display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px 10px 14px; border: 0; border-radius: 8px; cursor: pointer;
    background: var(--raised); color: var(--t-head); font-weight: 600; font-size: 15px; transition: background .15s;
  }
  .demo :global(.icon) { color: var(--acc-text); }
  .demo:hover { background: #43454b; }
  .local { display: inline-flex; align-items: center; gap: 8px; font-size: 14px; color: var(--t-muted); }

  .band { max-width: 1320px; margin: 0 auto; padding: 32px 24px; }
  .band h2 { margin: 0 0 16px; font: 900 24px/1.2 var(--display); color: var(--t-head); }

  .steps { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; counter-reset: step; }
  .steps li { display: flex; align-items: center; gap: 14px; padding: 18px 20px; border-radius: var(--radius); background: var(--card); counter-increment: step; }
  .step { position: relative; width: 44px; height: 44px; flex: none; border-radius: 50%; display: grid; place-items: center; background: var(--raised); color: var(--t-head); }
  .step::after {
    content: counter(step); position: absolute; right: -4px; bottom: -4px; width: 20px; height: 20px; border-radius: 50%;
    display: grid; place-items: center; background: var(--acc-strong); color: var(--on-acc); font: 800 11px/1 var(--ui); box-shadow: 0 0 0 3px var(--card);
  }
  .steps p { margin: 0; color: var(--t-body); line-height: 1.7; }
  .path { display: inline-flex; align-items: center; gap: 4px; flex-wrap: wrap; }
  .path :global(.icon) { color: var(--t-muted); }
  .key { padding: 2px 8px; border-radius: 6px; background: var(--raised); color: var(--t-head); font-weight: 600; white-space: nowrap; }

  .faq { display: grid; gap: 8px; }
  .faq h2 { margin-bottom: 8px; }
  details { border-radius: var(--radius); background: var(--card); }
  summary {
    display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 14px 18px;
    cursor: pointer; list-style: none; font-weight: 600; color: var(--t-head);
  }
  summary::-webkit-details-marker { display: none; }
  summary :global(.icon) { color: var(--t-muted); transition: rotate .2s; }
  details[open] summary :global(.icon) { rotate: 180deg; }
  details p { margin: 0; padding: 0 18px 16px; color: var(--t-body); max-width: 70em; }
  details a { color: var(--acc-text); font-weight: 600; text-underline-offset: 3px; }

  footer { max-width: 1320px; margin: 0 auto; padding: 24px 24px 48px; color: var(--t-faint); font-size: 13px; }
  .credit { color: inherit; text-decoration: none; white-space: nowrap; transition: color .15s; }
  .credit:hover, .credit:focus-visible { color: var(--acc-text); text-decoration: underline; }
  .sep { margin: 0 .4em; }

  @media (max-width: 1100px) {
    .in { grid-template-columns: minmax(0, 1fr); gap: 40px; padding-top: 56px; }
    .hero :global(.wall) { mask-image: linear-gradient(to bottom, rgba(0, 0, 0, .3) 0%, rgba(0, 0, 0, .12) 30%, rgba(0, 0, 0, .12) 42%, #000 62%, transparent 98%); mask-composite: add; }
    .steps { grid-template-columns: minmax(0, 1fr); }
  }
  @media (max-width: 620px) {
    .in { padding: 40px 16px 48px; }
    .band { padding: 24px 16px; }
    .lede { font-size: 17px; margin-bottom: 24px; }
    .drop { flex-wrap: wrap; }
    .choose { width: 100%; text-align: center; }
    footer { padding: 16px 16px 40px; }
  }
</style>
