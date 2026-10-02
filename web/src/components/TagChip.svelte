<script>
  // A server tag, as Discord shows it next to a name: the server's little badge and its tag.
  // With `badgeHash` (friends' tags in the account file) the real badge image is used. Tag events
  // only name the badge's shape and its two colours, so those are drawn as Discord's pixel art.
  import { tagBadgeUrl } from "../lib/discord.js";
  import { badgePixels } from "../lib/tagBadges.js";

  let { tag, badge = null, colours = [], guild = "", badgeHash = null, tip = "", sub = "" } = $props();

  const pixels = $derived(badgePixels(badge, colours[0], colours[1]));
  let failed = $state(false);
</script>

<span class="chip" data-tip={tip || null} data-sub={sub || null}>
  {#if badgeHash && !failed}
    <img src={tagBadgeUrl(guild, badgeHash)} alt="" width="16" height="16" onerror={() => (failed = true)} />
  {:else if pixels}
    <svg viewBox="0 0 16 16" width="16" height="16" shape-rendering="crispEdges" aria-hidden="true">
      {#each pixels as r}<rect x={r.x} y={r.y} width={r.w} height="1" fill={r.fill} />{/each}
    </svg>
  {:else}
    <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
      <circle cx="8" cy="8" r="6.4" fill={colours[0] || "#949ba4"} />
      <circle cx="8" cy="8" r="2.6" fill={colours[1] || "#dbdee1"} />
    </svg>
  {/if}
  <b>{tag}</b>
</span>

<style>
  .chip {
    display: inline-flex; align-items: center; gap: 4px; vertical-align: middle;
    padding: 2px 6px 2px 4px; border-radius: 6px; background: var(--raised);
    font: 700 12px/1.3 var(--ui); color: var(--t-head); letter-spacing: .02em; white-space: nowrap;
  }
  img, svg { display: block; flex: none; image-rendering: pixelated; }
</style>
