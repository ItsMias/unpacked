<script>
  // A server's icon, or its initials on a grey squircle the way Discord shows icon-less servers.
  let { url = null, name = "", size = 40 } = $props();

  let failed = $state(false);
  const initials = $derived(
    name
      .replace(/[^\p{L}\p{N}\s]/gu, " ")
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 3)
      .map((w) => [...w][0])
      .join("") || "?",
  );
</script>

{#if url && !failed}
  <img src={url} alt="" width={size} height={size} style:width="{size}px" style:height="{size}px" loading="lazy" onerror={() => (failed = true)} />
{:else}
  <span class="ini" style:width="{size}px" style:height="{size}px" style:font-size="{Math.round(size * (initials.length > 2 ? 0.28 : 0.36))}px">{initials}</span>
{/if}

<style>
  img, .ini { display: block; flex: none; border-radius: 30%; object-fit: cover; }
  .ini { display: grid; place-items: center; background: var(--raised); color: var(--t-body); font-weight: 600; }
</style>
