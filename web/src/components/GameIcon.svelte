<script>
  // A game's icon from Discord's public app info, with its name in the tooltip.
  import { appInfo } from "../lib/discord.js";

  let { id, size = 44, sub = "" } = $props();

  let info = $state(null);
  $effect(() => {
    let live = true;
    appInfo(id).then((a) => live && (info = a));
    return () => (live = false);
  });
</script>

<span class="game" style:width="{size}px" style:height="{size}px" data-tip={info?.name ?? "Game"} data-sub={sub || null}>
  {#if info?.icon}<img src={info.icon} alt={info.name} width={size} height={size} loading="lazy" />{/if}
</span>

<style>
  .game { display: block; flex: none; border-radius: 10px; overflow: hidden; background: var(--raised); }
  img { display: block; width: 100%; height: 100%; object-fit: cover; }
</style>
