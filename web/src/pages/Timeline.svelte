<script>
  import Avatar from "../components/Avatar.svelte";
  import Emoji from "../components/Emoji.svelte";
  import Icon from "../components/Icon.svelte";
  import RichText from "../components/RichText.svelte";
  import TagChip from "../components/TagChip.svelte";
  import { appInfo } from "../lib/discord.js";
  import { flag } from "../lib/names.js";
  import { fmt, longDate, timeline } from "../lib/stats.js";

  let { result, view, range, games } = $props();

  const items = $derived(timeline(view, result.facts, games, result).filter((x) => range === "all" || new Date(x.ms).getUTCFullYear() === range));
  const years = $derived.by(() => {
    const out = [];
    for (const it of items) {
      const y = new Date(it.ms).getUTCFullYear();
      if (out.at(-1)?.year !== y) out.push({ year: y, items: [] });
      out.at(-1).items.push(it);
    }
    return out;
  });
  // Years you've folded away.
  let closed = $state({});
  const day = (ms) => longDate(new Date(ms).toISOString().slice(0, 10));
  const short = (t) => (t.length > 280 ? `${t.slice(0, 280)}…` : t);

  // Names and icons of the Activities mentioned.
  let apps = $state({});
  $effect(() => {
    for (const it of items) if (it.app && !(it.app in apps)) appInfo(it.app).then((a) => (apps = { ...apps, [it.app]: a }));
  });

  // Discord's badge colours, tier 1 to 10, for the Account Age and Streaming milestones.
  const TIER = {
    age: ["#8fa9c4", "#6dc56d", "#43c07a", "#2bb3a6", "#36a8e0", "#4a83f0", "#6c6cf0", "#9a5ef0", "#e05cb8", "#f2a83b"],
    stream: ["#9cc3e6", "#74c96f", "#4fc46f", "#2cb7a2", "#2fa8dc", "#3f86f2", "#8a6cf0", "#e45aa8", "#ea4f78", "#f5a623"],
  };
</script>

<div class="page">
  {#each years as y (y.year)}
    <section class="year">
      <h3>
        <button aria-expanded={!closed[y.year]} onclick={() => (closed[y.year] = !closed[y.year])}>
          {y.year}<small>{fmt(y.items.length)} milestone{y.items.length === 1 ? "" : "s"}</small>
          <span class="chev" class:shut={closed[y.year]}><Icon name="down" size={20} stroke={2.5} /></span>
        </button>
      </h3>
      {#if !closed[y.year]}
        <ol>
          {#each y.items as it (it.ms + it.title)}
            {@const app = it.app ? apps[it.app] : null}
            <li>
              {#if it.person}
                <span class="icon pic"><Avatar id={it.person.id} hash={it.person.avatar} name={it.person.display_name} size={36} /></span>
              {:else if it.flag}
                <span class="icon"><Emoji text={flag(it.flag)} size={20} /></span>
              {:else if it.tier}
                <span class="icon tier" style:background={TIER[it.tier.kind][it.tier.n - 1]}>
                  {#if it.tier.kind === "age"}{it.tier.n}{:else}<Icon name="stream" size={18} />{/if}
                </span>
              {:else if app?.icon}
                <span class="icon pic"><img src={app.icon} alt="" width="36" height="36" /></span>
              {:else}
                <span class="icon"><Icon name={it.icon} size={18} /></span>
              {/if}
              <div class="body">
                <time>{day(it.ms)}</time>
                <strong>{it.title}</strong>
                {#if it.tag}<span class="tagline"><TagChip tag={it.tag.tag} badge={it.tag.badge} colours={it.tag.colours} />{#if it.sub}<small>{it.sub}</small>{/if}</span>
                {:else if it.sub || app?.name}<small>{it.sub || app.name}</small>{/if}
                {#if it.quote}<blockquote><RichText text={short(it.quote)} people={view.people} /></blockquote>{/if}
                {#if it.image}<img class="photo" src={URL.createObjectURL(it.image)} alt="" width="56" height="56" />{/if}
              </div>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  {:else}
    <p class="muted">Nothing happened in this range.</p>
  {/each}
</div>

<style>
  .year { position: relative; margin-top: 16px; }
  h3 { position: sticky; top: 61px; z-index: 2; margin: 0 0 8px; background: var(--bg); }
  h3 button {
    display: flex; align-items: center; gap: 12px; width: 100%; padding: 8px 0; border: 0; background: none; cursor: pointer;
    font: 900 22px/1 var(--display); color: var(--t-head); text-align: left;
  }
  h3 small { font: 600 13px var(--ui); color: var(--t-muted); }
  .chev { margin-left: auto; display: grid; color: var(--t-muted); transition: rotate .2s; }
  .chev.shut { rotate: -90deg; }
  h3 button:hover .chev { color: var(--t-head); }
  ol { list-style: none; margin: 0 0 8px; padding: 0 0 0 20px; border-left: 2px solid var(--line); display: grid; gap: 10px; }
  li { position: relative; display: grid; grid-template-columns: 36px 1fr; gap: 14px; align-items: start; margin-left: -39px; }
  .icon {
    width: 36px; height: 36px; border-radius: 50%; display: grid; place-items: center; overflow: hidden;
    background: var(--card); color: var(--acc-text); box-shadow: 0 0 0 4px var(--bg);
  }
  .icon.pic img { width: 36px; height: 36px; object-fit: cover; }
  .icon.tier { color: #fff; font: 900 16px/1 var(--display); text-shadow: 0 1px 2px rgba(0, 0, 0, .35); }
  .body { background: var(--card); border-radius: var(--radius); padding: 12px 16px; min-width: 0; }
  time { display: block; font-size: 12px; color: var(--t-muted); margin-bottom: 2px; }
  strong { display: block; color: var(--t-head); font-weight: 600; }
  small { display: block; color: var(--t-muted); font-size: 13px; margin-top: 2px; }
  .tagline { display: flex; align-items: center; gap: 8px; margin-top: 6px; }
  .tagline small { margin: 0; }
  blockquote {
    margin: 8px 0 0; padding: 8px 12px; border-left: 4px solid var(--acc); border-radius: 4px; background: var(--sunk);
    color: var(--t-body); white-space: pre-wrap; overflow-wrap: anywhere; font-size: 14px;
  }
  .photo { display: block; margin-top: 8px; width: 56px; height: 56px; border-radius: 50%; object-fit: cover; }
</style>
