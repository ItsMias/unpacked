<script>
  // One emoji, drawn like Discord draws it: Twemoji for Unicode emoji, the CDN image for custom
  // ones. Falls back to the plain character if the image can't load.
  import { emojiUrl } from "../lib/discord.js";
  import { twemojiUrl } from "../lib/twemoji.js";

  let { text = null, id = null, name = "", animated = false, size = 22 } = $props();

  let failed = $state(false);
  const src = $derived(id ? emojiUrl(id, animated) : text ? twemojiUrl(text) : null);
</script>

{#if src && !failed}
  <img class="emoji" {src} alt={text ?? `:${name}:`} width={size} height={size} style:width="{size}px" style:height="{size}px" loading="lazy" draggable="false" onerror={() => (failed = true)} />
{:else}
  <span class="emoji" style:font-size="{size * 0.9}px">{text ?? `:${name}:`}</span>
{/if}

<style>
  .emoji { display: inline-block; vertical-align: -0.2em; object-fit: contain; line-height: 1; }
</style>
