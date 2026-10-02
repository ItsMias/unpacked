<script>
  // A Discord avatar: a local image, the CDN copy, or initials on one of Discord's default colours.
  let { id = "", hash = null, src = null, name = "?", size = 36, decoration = null } = $props();

  const COLOURS = ["#5865f2", "#757e8a", "#3ba55c", "#faa61a", "#ed4245", "#eb459f"];
  let failed = $state(false);

  const url = $derived(src ?? (hash && id ? `https://cdn.discordapp.com/avatars/${id}/${hash}.${hash.startsWith("a_") ? "gif" : "png"}?size=${size > 64 ? 256 : 64}` : null));
  const colour = $derived(COLOURS[Number((BigInt(/^\d+$/.test(id) ? id : "0") >> 22n) % 6n)]);
</script>

<span class="av" style:width="{size}px" style:height="{size}px">
{#if url && !failed}
  <img src={url} alt="" width={size} height={size} style:width="{size}px" style:height="{size}px" loading="lazy" onerror={() => (failed = true)} />
{:else}
  <span class="ini" style:width="{size}px" style:height="{size}px" style:background={colour} style:font-size="{Math.round(size * 0.4)}px">
    {[...name][0]?.toUpperCase() ?? "?"}
  </span>
{/if}
{#if decoration}
  <img class="deco" src="https://cdn.discordapp.com/avatar-decoration-presets/{decoration}.png?size=96&passthrough=false" alt="" loading="lazy" />
{/if}
</span>

<style>
  .av { position: relative; display: block; flex: none; }
  .deco { position: absolute; inset: -10%; width: 120%; height: 120%; border-radius: 0; pointer-events: none; }
  img, .ini { border-radius: 50%; display: block; flex: none; }
  .ini { display: grid; place-items: center; color: #fff; font-weight: 700; }
</style>
