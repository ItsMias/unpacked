<script>
  // The home page's peek at the dashboard: the demo account's profile, clock and calendar as
  // they look on the Overview, with a few highlights floating around them.
  import Avatar from "./Avatar.svelte";
  import Emoji from "./Emoji.svelte";
  import Icon from "./Icon.svelte";
  import RadialHours from "./RadialHours.svelte";
  import TagChip from "./TagChip.svelte";
  import { badgeUrl, fmt, shortDate } from "../lib/stats.js";

  /** `sample`: summarize() of the demo, or null while it's being made. */
  let { sample } = $props();

  const hours = (ms) => {
    const h = ms / 3_600_000;
    return `${Math.floor(h)} h ${Math.round((h % 1) * 60)} min`;
  };
  const short = (key) => shortDate(Date.parse(`${key}T12:00:00Z`));
</script>

<div class="collage" aria-hidden="true">
  {#if sample}
    {@const p = sample.profile}
    <article class="card profile" style="--i: 0">
      <div class="banner"></div>
      <div class="pav"><Avatar id={p.id} name={p.display_name} size={72} /></div>
      <h3>{p.display_name}</h3>
      <div class="idline">
        <span>@{p.username}</span>
        {#if sample.tag}<TagChip tag={sample.tag.tag} badge={sample.tag.badge} colours={sample.tag.colours} />{/if}
        <span class="badges">{#each sample.badges as b (b.icon)}<img src={badgeUrl(b.icon)} alt="" width="18" height="18" />{/each}</span>
      </div>
      <dl>
        <div><dt>Messages sent</dt><dd class="num">{fmt(sample.o.sent)}</dd></div>
        <div><dt>Hours in voice</dt><dd class="num">{fmt(sample.o.voiceHours)}</dd></div>
      </dl>
    </article>

    <article class="card clock" style="--i: 1">
      <h3>When you talk</h3>
      <RadialHours hours={sample.clock} format={fmt} unit=" messages" />
      {#if sample.longest}
        <div class="toast call" style="--i: 3">
          <span class="ti"><Icon name="voice" size={18} /></span>
          <span><small>Longest call</small><b>{hours(sample.longest.end_ms - sample.longest.start_ms)}</b></span>
        </div>
      {/if}
      <div class="toast emo" style="--i: 5">
        <Emoji text={sample.emoji[0].text} size={26} />
        <span><small>Top emoji</small><b>{fmt(sample.emoji[0].count)} times</b></span>
      </div>
    </article>

    <article class="card heat" style="--i: 2">
      <h3>Messages per day <small>{sample.year}</small></h3>
      <div class="cal">
        {#each sample.cells as l, i (i)}<i class="cell" class:pad={l < 0} data-l={l}></i>{/each}
      </div>
      {#if sample.best}
        <div class="toast best" style="--i: 4">
          <span class="ti"><Icon name="fire" size={18} /></span>
          <span><small>Busiest day · {short(sample.best.key)}</small><b>{fmt(sample.best.n)} messages</b></span>
        </div>
      {/if}
    </article>
  {/if}
</div>

<style>
  .collage {
    width: 100%; max-width: 620px; min-height: 440px; margin-bottom: 24px; justify-self: end;
    display: grid; grid-template-columns: 1.15fr 1fr; gap: 16px; align-content: start;
  }
  .collage > *, .toast { animation: rise .7s cubic-bezier(.2, .8, .2, 1) both; animation-delay: calc(var(--i) * 110ms + 120ms); }
  @keyframes rise { from { opacity: 0; translate: 0 18px; scale: .96; } }

  .card { position: relative; box-shadow: 0 12px 32px rgba(0, 0, 0, .35); padding: 16px 18px; }
  h3 { margin: 0 0 12px; font-size: 15px; font-weight: 700; color: var(--t-head); }
  h3 small { font-size: 12px; font-weight: 500; color: var(--t-muted); margin-left: 4px; }

  .profile { padding: 0 18px 16px; overflow: hidden; }
  .banner {
    height: 64px; margin: 0 -18px;
    background:
      radial-gradient(120% 140% at 50% 120%, var(--acc) 0%, transparent 55%),
      radial-gradient(60% 90% at 50% 100%, rgba(255, 255, 255, .18) 0%, transparent 60%),
      linear-gradient(#0b023a, #1e0e54);
  }
  .pav { width: 84px; height: 84px; border-radius: 50%; border: 6px solid var(--card); background: var(--card); margin-top: -42px; }
  .profile h3 { font: 800 20px/1.2 var(--ui); margin: 4px 0 2px; }
  .idline { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; font-size: 14px; color: var(--t-body); }
  .badges { display: inline-flex; gap: 3px; }
  .badges img { display: block; width: 18px; height: 18px; }
  dl { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin: 14px 0 0; padding-top: 12px; border-top: 1px solid var(--line); }
  dt { font-size: 12px; font-weight: 600; color: var(--t-muted); }
  dd { margin: 2px 0 0; font: 900 20px/1.1 var(--display); color: var(--t-head); }

  /* Above the calendar, so its toasts can hang over it. */
  .clock { display: flex; flex-direction: column; z-index: 2; }
  .clock :global(svg) { max-width: 210px; margin: auto; }

  .heat { grid-column: 1 / -1; }
  .cal { display: grid; grid-template-rows: repeat(7, 1fr); grid-auto-flow: column; grid-auto-columns: 1fr; gap: 2px; }
  .cal i { aspect-ratio: 1; border-radius: 2px; }
  .cal .pad { visibility: hidden; }

  .toast {
    position: absolute; z-index: 2; display: flex; align-items: center; gap: 10px;
    padding: 10px 14px 10px 10px; border-radius: 12px; background: #111214;
    box-shadow: 0 10px 28px rgba(0, 0, 0, .45), 0 0 0 1px rgba(255, 255, 255, .04);
    white-space: nowrap;
  }
  .toast small { display: block; font-size: 12px; font-weight: 600; color: var(--t-muted); }
  .toast b { display: block; font-size: 15px; color: var(--t-head); }
  .ti { width: 34px; height: 34px; border-radius: 10px; display: grid; place-items: center; background: var(--acc-soft); color: var(--acc-text); }
  .call { top: -26px; right: -22px; }
  .emo { right: -22px; bottom: -30px; }
  .best { left: -40px; bottom: -42px; }

  @media (max-width: 1100px) {
    .collage { justify-self: start; min-height: 0; }
    .best { left: 16px; }
  }
  @media (max-width: 620px) {
    .collage { grid-template-columns: 1fr; margin-bottom: 32px; }
    .clock { display: none; }
  }
</style>
