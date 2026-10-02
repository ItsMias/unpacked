<script>
  // Accent colour picker. The choice is a per-browser preference, kept in localStorage.
  const OPTIONS = [
    { id: "magenta", label: "Magenta", colour: "#bd4be0" },
    { id: "pink", label: "Pink", colour: "#f5a9b8" },
    { id: "violet", label: "Violet", colour: "#8f7ff0" },
    { id: "green", label: "Green", colour: "#2ea043" },
  ];
  const KEY = "unpacked.accent";

  const stored = () => { try { return localStorage.getItem(KEY); } catch { return null; } };
  let accent = $state(OPTIONS.some((o) => o.id === stored()) ? stored() : "magenta");
  let open = $state(false);

  $effect(() => {
    document.documentElement.dataset.accent = accent;
    try { localStorage.setItem(KEY, accent); } catch { /* private mode: not remembered */ }
  });

  function onWindowClick(e) {
    if (open && !e.target.closest(".accent")) open = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="accent">
  <button class="trigger" aria-label="Accent colour" aria-expanded={open} onclick={() => (open = !open)}>
    <span class="dot" style:background={OPTIONS.find((o) => o.id === accent).colour}></span>
  </button>
  {#if open}
    <div class="menu" role="menu">
      <p>Accent</p>
      {#each OPTIONS as o}
        <button role="menuitemradio" aria-checked={accent === o.id} onclick={() => { accent = o.id; open = false; }}>
          <span class="dot" style:background={o.colour}></span>{o.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .accent { position: relative; }
  .trigger { width: 36px; height: 36px; border: 0; border-radius: 8px; background: var(--card); display: grid; place-items: center; cursor: pointer; }
  .trigger:hover { background: var(--raised); }
  .dot { width: 16px; height: 16px; border-radius: 50%; display: block; flex: none; }
  .menu {
    position: absolute; right: 0; top: calc(100% + 8px); z-index: 30; min-width: 160px;
    background: #111214; border-radius: 8px; padding: 6px; box-shadow: 0 8px 24px rgba(0, 0, 0, .35);
  }
  .menu p { margin: 4px 8px 6px; font-size: 12px; font-weight: 600; color: var(--t-muted); }
  .menu button {
    display: flex; align-items: center; gap: 10px; width: 100%; border: 0; background: none; cursor: pointer;
    padding: 8px; border-radius: 4px; font-size: 14px; font-weight: 500; color: var(--t-body); text-align: left;
  }
  .menu button:hover { background: var(--acc-strong); color: var(--on-acc); }
  .menu button[aria-checked="true"] { color: var(--t-head); font-weight: 700; }
</style>
