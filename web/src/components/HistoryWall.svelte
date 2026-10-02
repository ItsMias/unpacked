<script>
  // The loading screen's live calendar: every day of every year, filling in as the package is
  // read. Message days light up as their folders are read (cells pop when they appear or get
  // brighter). While the activity log streams, voice time drops in as dots that pop the same way,
  // and days where deleted messages turn up send out a ring. Game history, read alongside,
  // flashes the days you played. Drawn on a canvas; a few thousand cells are cheap at 30 fps.
  //   days:    Map<UTC day number, messages>
  //   voice:   Map<UTC day number, minutes in voice>
  //   deleted: Map<UTC day number, deleted messages (by you, a mod, or with their server)>
  //   played:  Map<UTC day number, minutes played>
  //   years:   [first, last]
  let { days, voice, deleted, played, years } = $props();

  const DAY = 86_400_000;
  const POP_MS = 420, RING_MS = 1000, FLASH_MS = 800;
  // Rings and flashes each update may start (more would light up everything at once), spread
  // over the time until the next update.
  const MAX_RINGS = 28, MAX_FLASHES = 34, SPREAD_MS = 350;
  // Voice dot radius per size step, as a share of the cell.
  const DOT = [0, 0.2, 0.26, 0.32, 0.38];
  let canvas = $state();
  let width = $state(0);
  let height = $state(0);

  // Per-day animation state.
  const shown = new Map(); // day -> message level drawn
  const popAt = new Map(); // day -> time the level went up
  const dots = new Map(); // day -> { from, to, at }: voice dot steps, and when it grew
  const ringAt = new Map(); // day -> when deleted messages turned up
  const flashAt = new Map(); // day -> when playtime turned up
  const seenDeleted = new Map(), seenPlayed = new Map();

  /** A level function: 0 for nothing, then one step more past each quantile of the values. */
  const levelsFor = (map, qs) => {
    const vals = [...map.values()].filter((n) => n > 0).sort((a, b) => a - b);
    const breaks = qs.map((p) => vals[Math.floor(p * (vals.length - 1))] ?? 0);
    return (n) => (n <= 0 ? 0 : 1 + breaks.filter((b) => n > b).length);
  };

  $effect(() => {
    const level = levelsFor(days, [0.3, 0.55, 0.78, 0.93]);
    const now = performance.now();
    for (const [d, n] of days) {
      const l = level(n);
      if (l > (shown.get(d) ?? 0)) popAt.set(d, now);
      shown.set(d, l);
    }
  });

  $effect(() => {
    const step = levelsFor(voice, [0.35, 0.65, 0.88]);
    const now = performance.now();
    for (const [d, m] of voice) {
      const l = step(m), dot = dots.get(d);
      if (!dot || l > dot.to) dots.set(d, { from: !dot ? 0 : now < dot.at ? dot.from : dot.to, to: l, at: now + Math.random() * SPREAD_MS });
      else if (l < dot.to) dots.set(d, { from: l, to: l, at: 0 });
    }
  });

  /** Starts rings or flashes on a sample of the days whose count went up since the last update. */
  function spark(map, seen, at, max) {
    const now = performance.now();
    const grew = [];
    for (const [d, n] of map) {
      if (n > (seen.get(d) ?? 0)) grew.push(d);
      seen.set(d, n);
    }
    for (let i = 0; i < Math.min(max, grew.length); i++) {
      const j = i + Math.floor(Math.random() * (grew.length - i));
      [grew[i], grew[j]] = [grew[j], grew[i]];
      at.set(grew[i], now + Math.random() * SPREAD_MS);
    }
  }
  $effect(() => spark(deleted, seenDeleted, ringAt, MAX_RINGS));
  $effect(() => spark(played, seenPlayed, flashAt, MAX_FLASHES));

  function colours() {
    const css = getComputedStyle(document.documentElement);
    const v = (k) => css.getPropertyValue(k).trim();
    return {
      h: [v("--h0"), v("--h1"), v("--h2"), v("--h3"), v("--h4"), v("--h5")],
      voice: v("--cat-2"), games: v("--cat-3"), ring: v("--t-head"),
      label: v("--t-muted"), total: v("--t-head"), font: v("--ui"),
    };
  }

  $effect(() => {
    if (!canvas || !width || !height) return;
    const g = canvas.getContext("2d");
    const dpr = Math.min(2, devicePixelRatio || 1);
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let c = colours();
    let lastColours = 0;
    let raf = 0, last = 0;
    let layout = null;

    // Where every day goes, worked out once per size and year span: rows of years, each a
    // 53-week × 7-day block with the year on the left and its totals on the right.
    function place(y0, y1) {
      const n = y1 - y0 + 1;
      const LABEL = 44, TOTAL = 72;
      const pitchW = (width - LABEL - TOTAL) / 53;
      const pitchH = height / (n * 7 + (n - 1) * 1.4);
      const p = Math.max(4, Math.min(pitchW, pitchH, 22));
      const size = p - (p < 9 ? 1.5 : 3);
      const ox = (width - (LABEL + 53 * p + TOTAL)) / 2 + LABEL;
      const oy = (height - (n * 7 * p + (n - 1) * 1.4 * p)) / 2;
      const rows = [];
      for (let year = y0; year <= y1; year++) {
        const top = oy + (year - y0) * p * 8.4;
        const first = Date.UTC(year, 0, 1);
        const lead = (new Date(first).getUTCDay() + 6) % 7;
        const cells = [];
        for (let t = first, k = lead; new Date(t).getUTCFullYear() === year; t += DAY, k++) {
          cells.push(t / DAY, ox + Math.floor(k / 7) * p + size / 2, top + (k % 7) * p + size / 2);
        }
        rows.push({ year, mid: top + 3.5 * p, cells });
      }
      return { y0, y1, p, size, ox, rows, font: Math.max(10, Math.min(13, p * 0.8)) };
    }

    const draw = (now) => {
      if (now - lastColours > 500) { c = colours(); lastColours = now; }
      if (layout?.y0 !== years[0] || layout?.y1 !== years[1]) layout = place(years[0], years[1]);
      const { p, size, ox, rows, font } = layout;
      const corner = Math.min(3, size / 4);
      // Settled cells and dots are batched into one path per colour; anything mid-animation is
      // drawn on its own on top.
      const cellPaths = c.h.map(() => new Path2D());
      const dotPath = new Path2D();
      const popping = [], flashes = [], growing = [], rings = [];

      g.clearRect(0, 0, width, height);
      g.textBaseline = "middle";
      for (const { year, mid, cells } of rows) {
        let total = 0, voiceMin = 0, playMin = 0;
        for (let i = 0; i < cells.length; i += 3) {
          const d = cells[i], cx = cells[i + 1], cy = cells[i + 2];
          total += days.get(d) ?? 0;
          voiceMin += voice.get(d) ?? 0;
          playMin += played.get(d) ?? 0;

          // The day's message cell, popping when it lights up or gets brighter.
          const lv = shown.get(d) ?? 0;
          const pop = popAt.get(d);
          const age = pop === undefined || reduce ? 1 : (now - pop) / POP_MS;
          if (age < 1) popping.push(cx, cy, lv, age);
          else cellPaths[lv].roundRect(cx - size / 2, cy - size / 2, size, size, corner);

          // Playtime turning up: a flash that swells past the cell and fades.
          const fl = flashAt.get(d);
          if (fl !== undefined && !reduce) {
            const a = (now - fl) / FLASH_MS;
            if (a >= 1) flashAt.delete(d);
            else if (a >= 0) flashes.push(cx, cy, a);
          }

          // Voice: a dot, popping when it appears or grows.
          const dot = dots.get(d);
          if (dot) {
            const grown = reduce || now >= dot.at;
            const da = grown && !reduce ? (now - dot.at) / POP_MS : 1;
            const r = Math.max(1.2, DOT[grown ? dot.to : dot.from] * size);
            if (da < 1) growing.push(cx, cy, r * (1 + (1 - da) ** 2 * 0.9), da);
            else if (DOT[grown ? dot.to : dot.from]) { dotPath.moveTo(cx + r, cy); dotPath.arc(cx, cy, r, 0, 2 * Math.PI); }
          }

          // Deleted messages turning up: a ring.
          const rg = ringAt.get(d);
          if (rg !== undefined && !reduce) {
            const a = (now - rg) / RING_MS;
            if (a >= 1) ringAt.delete(d);
            else if (a >= 0) rings.push(cx, cy, a);
          }
        }

        // Year label on the left; the year's totals on the right as they come in.
        g.globalAlpha = 1;
        g.font = `600 ${font}px ${c.font}`;
        g.fillStyle = c.label;
        g.textAlign = "right";
        g.fillText(String(year), ox - 10, mid);
        const lines = [[total ? total.toLocaleString("en-US") : "–", total ? c.total : c.label]];
        if (voiceMin >= 60) lines.push([`${Math.round(voiceMin / 60).toLocaleString("en-US")} h`, c.voice]);
        if (playMin >= 60) lines.push([`${Math.round(playMin / 60).toLocaleString("en-US")} h`, c.games]);
        const lh = font + 3;
        const shownLines = 7 * p >= lines.length * lh ? lines : lines.slice(0, 1);
        g.textAlign = "left";
        shownLines.forEach(([text, colour], k) => {
          g.fillStyle = colour;
          g.font = `${k ? 700 : 600} ${k ? font - 1 : font}px ${c.font}`;
          g.fillText(text, ox + 53 * p + 10, mid + (k - (shownLines.length - 1) / 2) * lh);
        });
      }

      g.globalAlpha = 1;
      cellPaths.forEach((path, lv) => { g.fillStyle = c.h[lv]; g.fill(path); });
      for (let i = 0; i < popping.length; i += 4) {
        const [cx, cy, lv, age] = [popping[i], popping[i + 1], popping[i + 2], popping[i + 3]];
        const s = size * (1 + (1 - age) ** 2 * 0.6);
        g.beginPath();
        g.roundRect(cx - s / 2, cy - s / 2, s, s, Math.min(3, s / 4));
        g.globalAlpha = 1;
        g.fillStyle = c.h[lv];
        g.fill();
        g.globalAlpha = (1 - age) * 0.7;
        g.fillStyle = "#fff";
        g.fill();
      }
      g.fillStyle = c.games;
      for (let i = 0; i < flashes.length; i += 3) {
        const [cx, cy, a] = [flashes[i], flashes[i + 1], flashes[i + 2]];
        const s = size * (1 + (1 - a) ** 2 * 0.45);
        g.globalAlpha = (1 - a) ** 1.5 * 0.9;
        g.beginPath();
        g.roundRect(cx - s / 2, cy - s / 2, s, s, Math.min(3, s / 4));
        g.fill();
      }
      g.globalAlpha = 1;
      g.fillStyle = c.voice;
      g.fill(dotPath);
      for (let i = 0; i < growing.length; i += 4) {
        const [cx, cy, r, da] = [growing[i], growing[i + 1], growing[i + 2], growing[i + 3]];
        g.beginPath();
        g.arc(cx, cy, r, 0, 2 * Math.PI);
        g.globalAlpha = 1;
        g.fillStyle = c.voice;
        g.fill();
        g.globalAlpha = (1 - da) * 0.6;
        g.fillStyle = "#fff";
        g.fill();
      }
      g.lineWidth = Math.max(1, size * 0.14);
      g.strokeStyle = c.ring;
      for (let i = 0; i < rings.length; i += 3) {
        const a = rings[i + 2];
        g.globalAlpha = (1 - a) ** 1.4 * 0.85;
        g.beginPath();
        g.arc(rings[i], rings[i + 1], size * (0.45 + a * 1.3), 0, 2 * Math.PI);
        g.stroke();
      }
      g.globalAlpha = 1;
    };

    // ~30 fps is plenty for these animations, and leaves the CPU to the workers reading the zip.
    const frame = (now) => {
      if (now - last > 32) { draw(now); last = now; }
      raf = requestAnimationFrame(frame);
    };
    // With reduced motion there's no loop: drawing here makes this effect re-run (and redraw)
    // whenever the data or the years change.
    if (reduce) draw(performance.now());
    else raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="wall" bind:clientWidth={width} bind:clientHeight={height}>
  <canvas bind:this={canvas} style:width="{width}px" style:height="{height}px" aria-hidden="true"></canvas>
</div>

<style>
  .wall { position: relative; width: 100%; height: 100%; min-height: 240px; }
  canvas { position: absolute; inset: 0; display: block; }
</style>
