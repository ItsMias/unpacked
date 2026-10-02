<script>
  import AccentMenu from "./AccentMenu.svelte";
  import Icon from "./Icon.svelte";
  import Select from "./Select.svelte";
  import { REPO } from "../lib/site.js";

  /**
   * `pages` is [{ id, label }]; without it (the drop screen) only the brand and accent show.
   * `ranges` is the options for the date range dropdown.
   */
  let { pages = null, page = "", ranges = [], range = $bindable("all"), status = "", demo = false, onopen = null } = $props();

  // Pages marked `more` sit in a menu at the end of the tabs.
  const main = $derived(pages?.filter((p) => !p.more) ?? []);
  const more = $derived(pages?.filter((p) => p.more) ?? []);
  const inMore = $derived(more.some((p) => p.id === page));
  let open = $state(false);
  // The tabs scroll sideways on small screens, which would clip a normal dropdown, so the menu
  // is placed with fixed coordinates under its button.
  let at = $state({ x: 0, y: 0 });
  function toggle(e) {
    const r = e.currentTarget.getBoundingClientRect();
    at = { x: Math.min(r.left, innerWidth - 216), y: r.bottom + 8 };
    open = !open;
  }
</script>


<svelte:window
  onclick={(e) => open && !e.target.closest(".more") && (open = false)}
  onkeydown={(e) => e.key === "Escape" && (open = false)}
  onscroll={() => (open = false)}
  onresize={() => (open = false)}
/>

<header class="top">
  <div class="in">
    <a class="brand" href="#/"><span class="mark" aria-hidden="true">U</span>Unpacked</a>
    {#if demo}<span class="demo" title="Made-up data, not a real account">Demo</span>{/if}
    {#if pages}
      <nav class="tabs" aria-label="Sections">
        {#each main as p}
          <a class="tab" href="#/{p.id}" aria-current={p.id === page ? "page" : undefined}>{p.label}</a>
        {/each}
        {#if more.length}
          <div class="more">
            <button class="tab" class:on={inMore} aria-expanded={open} aria-haspopup="true" onclick={toggle}>
              More
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
            </button>
            {#if open}
              <div class="menu" style:left="{at.x}px" style:top="{at.y}px">
                {#each more as p}
                  <a href="#/{p.id}" aria-current={p.id === page ? "page" : undefined} onclick={() => (open = false)}>{p.label}</a>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </nav>
    {/if}
    <div class="actions">
      {#if status}<span class="status" aria-live="polite">{status}</span>{/if}
      {#if pages}<Select options={ranges} bind:value={range} label="Date range" />{/if}
      <!-- On the home page it says so; next to the dashboard's tabs it's just the mark. -->
      <a class="source" class:wide={!pages} href={REPO} target="_blank" rel="noopener" aria-label={pages ? "Source code on GitHub" : "Open source on GitHub"} data-tip={pages ? "Source code on GitHub" : null}>
        <Icon name="github" size={18} />{#if !pages}<span>Open source</span>{/if}
      </a>
      <AccentMenu />
      {#if onopen}<button class="cta" onclick={onopen}>{demo ? "Open your package" : "Open package"}</button>{/if}
    </div>
  </div>
</header>

<style>
  .top { position: sticky; top: 0; z-index: 20; background: rgba(30, 31, 34, .92); backdrop-filter: blur(8px); border-bottom: 1px solid #2a2b30; }
  .in { max-width: 1320px; margin: 0 auto; padding: 0 24px; height: 60px; display: flex; align-items: center; gap: 28px; }
  .brand { display: flex; align-items: center; gap: 10px; font: 900 18px/1 var(--display); color: var(--t-head); text-decoration: none; letter-spacing: -.01em; }
  .mark { width: 30px; height: 30px; border-radius: 9px; background: var(--acc-strong); color: var(--on-acc); display: grid; place-items: center; font-size: 15px; }
  .demo {
    margin-left: -18px; padding: 3px 8px; border-radius: 4px; background: var(--acc-soft); color: var(--acc-text);
    font-size: 12px; font-weight: 800; letter-spacing: .04em; text-transform: uppercase;
  }
  .tabs { display: flex; gap: 4px; overflow-x: auto; scrollbar-width: none; }
  .tab { padding: 6px 12px; border-radius: 6px; color: var(--t-muted); font-weight: 600; font-size: 15px; white-space: nowrap; text-decoration: none; }
  .tab:hover { background: var(--hover); color: var(--t-body); }
  .tab[aria-current="page"], .tab.on { background: var(--raised); color: var(--t-head); }
  button.tab { border: 0; background: none; cursor: pointer; display: flex; align-items: center; gap: 6px; font: inherit; font-weight: 600; font-size: 15px; }
  .more { position: relative; }
  .menu {
    position: fixed; z-index: 40; min-width: 200px; padding: 6px;
    background: #111214; border-radius: 8px; box-shadow: 0 8px 24px rgba(0, 0, 0, .35); display: grid;
  }
  .menu a { padding: 8px 10px; border-radius: 4px; color: var(--t-body); text-decoration: none; font-size: 14px; font-weight: 500; }
  .menu a:hover { background: var(--acc-strong); color: var(--on-acc); }
  .menu a[aria-current="page"] { color: var(--t-head); font-weight: 700; }
  .actions { margin-left: auto; display: flex; align-items: center; gap: 8px; }
  .status { font-size: 13px; color: var(--t-muted); white-space: nowrap; display: flex; align-items: center; gap: 8px; }
  .status::before { content: ""; width: 8px; height: 8px; border-radius: 50%; background: var(--acc); animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: .3; } }
  .source {
    height: 36px; min-width: 36px; display: inline-flex; align-items: center; justify-content: center; gap: 8px;
    border-radius: 8px; background: var(--card); color: var(--t-body); text-decoration: none; font-size: 14px; font-weight: 600;
  }
  .source.wide { padding: 0 12px 0 10px; }
  .source:hover, .source:focus-visible { background: var(--raised); color: var(--t-head); }
  .cta { border: 0; border-radius: 6px; padding: 8px 14px; font-weight: 600; font-size: 14px; cursor: pointer; white-space: nowrap; background: var(--acc-strong); color: var(--on-acc); }
  .cta:hover { filter: brightness(1.1); }

  @media (max-width: 1180px) {
    .in { flex-wrap: wrap; height: auto; padding: 10px 24px; gap: 10px 16px; }
    .tabs { order: 3; width: 100%; }
  }
  @media (max-width: 620px) {
    .in { padding: 10px 16px; }
  }
  @media (max-width: 640px) {
    .status { display: none; }
  }
  @media (max-width: 480px) {
    .cta { display: none; }
  }
</style>
