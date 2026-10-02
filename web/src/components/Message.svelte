<script>
  // A message (or a run of them) laid out like in Discord: avatar, name, time, where, then text.
  //   parts: [{ ms, text }] — one entry for a single message, several for a run
  import Avatar from "./Avatar.svelte";
  import RichText from "./RichText.svelte";

  let { author, parts, place = "", tz = undefined, people = new Map(), cut = false } = $props();

  const stamp = (ms) => new Date(ms).toLocaleString("en-GB", { dateStyle: "short", timeStyle: "short", timeZone: tz });
  const clock = (ms) => new Date(ms).toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit", timeZone: tz });
</script>

<article class="msg">
  <Avatar id={author.id} src={author.src} hash={author.avatar} name={author.name} size={40} />
  <div class="body">
    <header>
      <strong>{author.name}</strong>
      <time>{stamp(parts[0].ms)}</time>
      {#if place}<span class="place">{place}</span>{/if}
    </header>
    {#each parts as p, i (i)}
      <div class="line">
        {#if i > 0}<span class="t">{clock(p.ms)}</span>{/if}
        <RichText text={p.text} {people} />{#if cut && i === parts.length - 1}<span class="more">…</span>{/if}
      </div>
    {/each}
  </div>
</article>

<style>
  .msg { display: grid; grid-template-columns: 40px 1fr; gap: 14px; }
  .body { min-width: 0; }
  header { display: flex; align-items: baseline; flex-wrap: wrap; gap: 4px 8px; margin-bottom: 2px; }
  strong { color: var(--t-head); font-weight: 600; }
  time { color: var(--t-faint); font-size: 12px; }
  .place { font-size: 12px; font-weight: 600; color: var(--t-muted); background: var(--sunk); border-radius: 4px; padding: 1px 6px; }
  .line { position: relative; color: var(--t-body); line-height: 1.375; }
  .line + .line { margin-top: 2px; }
  /* Like Discord, later messages in a run only show their time on hover. */
  .t { position: absolute; left: -54px; top: 2px; width: 44px; text-align: right; font-size: 11px; color: var(--t-faint); opacity: 0; }
  .line:hover .t { opacity: 1; }
  .line:hover { background: rgba(2, 2, 2, .06); }
  .more { color: var(--t-muted); }
</style>
