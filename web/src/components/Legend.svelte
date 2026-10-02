<div class="legend" aria-hidden="true">
  <span>Less</span>
  {#each [0, 1, 2, 3, 4, 5] as l}<i class="cell" data-l={l}></i>{/each}
  <span>More</span>
</div>

<style>
  .legend { display: flex; align-items: center; gap: 4px; font-size: 12px; color: var(--t-muted); }
  .cell { width: 11px; height: 11px; }
  span:first-child { margin-right: 4px; }
  span:last-child { margin-left: 4px; }
</style>
