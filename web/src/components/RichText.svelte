<script>
  // Message text as Discord shows it: emoji as images, custom emoji, and mentions as pills.
  import Emoji from "./Emoji.svelte";
  import { splitEmoji } from "../lib/twemoji.js";

  /** `people`: Map of user id -> { display_name }, for naming mentions. */
  let { text, people = new Map() } = $props();

  // <a:name:id>, <:name:id>, <@id>, <@!id>, <@&id>, <#id>
  const TOKEN = /<(a?):(\w+):(\d+)>|<@!?(\d+)>|<@&(\d+)>|<#(\d+)>/g;

  const parts = $derived.by(() => {
    const out = [];
    let at = 0;
    for (const m of text.matchAll(TOKEN)) {
      if (m.index > at) out.push(...splitEmoji(text.slice(at, m.index)));
      if (m[3]) out.push({ custom: { id: m[3], name: m[2], animated: m[1] === "a" } });
      else if (m[4]) out.push({ mention: `@${people.get(m[4])?.display_name ?? "someone"}` });
      else if (m[5]) out.push({ mention: "@role" });
      else out.push({ mention: "#channel" });
      at = m.index + m[0].length;
    }
    if (at < text.length) out.push(...splitEmoji(text.slice(at)));
    return out;
  });
  // Messages that are only emoji show them big, like Discord does.
  const jumbo = $derived(parts.length <= 27 && parts.every((p) => p.emoji || p.custom || (p.text && !p.text.trim())));
</script>

<span class="rich" class:jumbo>{#each parts as p, i (i)}{#if p.emoji}<Emoji text={p.emoji} size={jumbo ? 48 : 22} />{:else if p.custom}<Emoji id={p.custom.id} name={p.custom.name} animated={p.custom.animated} size={jumbo ? 48 : 22} />{:else if p.mention}<span class="mention">{p.mention}</span>{:else}{p.text}{/if}{/each}</span>

<style>
  .rich { white-space: pre-wrap; overflow-wrap: anywhere; }
  .mention { background: var(--acc-soft); color: var(--acc-text); border-radius: 3px; padding: 0 2px; font-weight: 500; }
</style>
