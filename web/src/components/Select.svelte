<script>
  // A dropdown in the site's own style (the native <select> menu can't be styled). Follows the
  // ARIA listbox pattern: arrow keys, Home/End, Enter/Space to pick, Escape to close.
  let { options, value = $bindable(), label } = $props();

  let open = $state(false);
  let active = $state(0);
  let button = $state();
  let list = $state();
  const id = `sel-${Math.random().toString(36).slice(2, 8)}`;

  const current = $derived(options.find((o) => o.value === value) ?? options[0]);

  function show() {
    active = Math.max(0, options.findIndex((o) => o.value === value));
    open = true;
    queueMicrotask(() => list?.focus());
  }
  function pick(i) {
    value = options[i].value;
    open = false;
    button?.focus();
  }
  function onListKey(e) {
    const last = options.length - 1;
    if (e.key === "ArrowDown") active = Math.min(last, active + 1);
    else if (e.key === "ArrowUp") active = Math.max(0, active - 1);
    else if (e.key === "Home") active = 0;
    else if (e.key === "End") active = last;
    else if (e.key === "Enter" || e.key === " ") pick(active);
    else if (e.key === "Escape" || e.key === "Tab") { open = false; if (e.key === "Escape") button?.focus(); return; }
    else return;
    e.preventDefault();
    list?.querySelector(`[data-i="${active}"]`)?.scrollIntoView({ block: "nearest" });
  }
  function onButtonKey(e) {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) { e.preventDefault(); show(); }
  }
</script>

<svelte:window onclick={(e) => open && !e.target.closest(`#${id}`) && (open = false)} />

<div class="select" {id}>
  <button
    bind:this={button}
    class="trigger"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label="{label}: {current.label}"
    onclick={() => (open ? (open = false) : show())}
    onkeydown={onButtonKey}
  >
    {current.label}
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
  </button>
  {#if open}
    <ul
      bind:this={list}
      class="menu"
      role="listbox"
      tabindex="-1"
      aria-label={label}
      aria-activedescendant="{id}-{active}"
      onkeydown={onListKey}
    >
      {#each options as o, i (o.value)}
        <!-- svelte-ignore a11y_click_events_have_key_events (keys are handled on the listbox) -->
        <li
          id="{id}-{i}"
          data-i={i}
          role="option"
          aria-selected={o.value === value}
          class:active={i === active}
          onclick={() => pick(i)}
          onmousemove={() => (active = i)}
        >
          <span>{o.label}</span>
          {#if o.note}<small class="num">{o.note}</small>{/if}
          {#if o.value === value}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M5 12.5 10 17l9-10" /></svg>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select { position: relative; }
  .trigger {
    display: flex; align-items: center; gap: 8px; border: 0; border-radius: 6px; padding: 8px 12px 8px 14px;
    font-weight: 600; font-size: 14px; cursor: pointer; white-space: nowrap; background: var(--card); color: var(--t-head);
  }
  .trigger:hover, .trigger[aria-expanded="true"] { background: var(--raised); }
  .trigger svg { color: var(--t-muted); transition: transform .15s; }
  .trigger[aria-expanded="true"] svg { transform: rotate(180deg); }

  .menu {
    position: absolute; right: 0; top: calc(100% + 8px); z-index: 30; min-width: 200px; max-height: 360px; overflow-y: auto;
    margin: 0; padding: 6px; list-style: none; background: #111214; border-radius: 8px; box-shadow: 0 8px 24px rgba(0, 0, 0, .35);
  }
  .menu:focus-visible { outline: none; }
  li {
    display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 4px; cursor: pointer;
    font-size: 14px; font-weight: 500; color: var(--t-body);
  }
  li span { flex: 1; }
  li small { color: var(--t-faint); font-size: 12px; }
  li svg { color: var(--acc-text); }
  li[aria-selected="true"] { color: var(--t-head); font-weight: 700; }
  li.active { background: var(--acc-strong); color: var(--on-acc); }
  li.active small, li.active svg { color: inherit; opacity: .85; }
</style>
