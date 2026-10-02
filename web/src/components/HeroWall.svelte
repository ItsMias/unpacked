<script>
  // The home page's background: a wall of calendar cells like the loading screen's, quietly busy.
  // A wave of activity rolls across and single days softly light up and fade, the way a message
  // history looks when it fills in. Brightness fades smoothly between the calendar's shades.
  // Drawn on a canvas at ~30 fps, redrawing only the cells that changed, and only while on screen.
  let canvas = $state();
  let width = $state(0);
  let height = $state(0);

  // The resting pattern: a few sine waves make soft clusters, so it reads like real activity.
  const field = (c, r) => (Math.sin(c * 0.31 + r * 0.17) + Math.sin(c * 0.07 - r * 0.41 + 1.3) + Math.sin((c + r) * 0.13 + 4.1)) / 3;

  /** The calendar shades --h0 to --h5 as [r, g, b]. */
  function shades() {
    const css = getComputedStyle(document.documentElement);
    return ["--h0", "--h1", "--h2", "--h3", "--h4", "--h5"].map((k) => {
      const m = /^#([0-9a-f]{6})$/i.exec(css.getPropertyValue(k).trim());
      return m ? [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16)) : [57, 59, 65];
    });
  }

  // Brightness is kept in eighths of a shade step, so fades look smooth but a still cell isn't
  // redrawn every frame.
  const STEPS = 8;

  $effect(() => {
    if (!canvas || !width || !height) return;
    const g = canvas.getContext("2d");
    const dpr = Math.min(2, devicePixelRatio || 1);
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;

    const P = width < 700 ? 20 : 26;
    const S = P - 4, R = P < 24 ? 4 : 5;
    const cols = Math.ceil(width / P) + 1, rows = Math.ceil(height / P) + 1;
    const ox = Math.round((width - (cols * P - (P - S))) / 2);
    const base = new Float32Array(cols * rows);
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) base[r * cols + c] = Math.max(0, field(c, r) * 1.7 + Math.random() * 0.9 - 0.1);
    }
    const glow = new Float32Array(cols * rows);
    const drawn = new Int16Array(cols * rows).fill(-1);
    let pops = [];
    let palette = shades(), lastShades = 0, paletteKey = palette.join();
    // Every colour a cell can take, worked out once per palette.
    let fills = [];
    const mixAll = () => {
      fills = [];
      for (let k = 0; k <= 5 * STEPS; k++) {
        const lo = Math.min(4, Math.floor(k / STEPS)), f = k / STEPS - lo;
        fills.push(`rgb(${palette[lo].map((v, i) => Math.round(v + (palette[lo + 1][i] - v) * f)).join(" ")})`);
      }
    };
    mixAll();
    let raf = 0, last = 0, visible = true;

    const draw = (now) => {
      if (now - lastShades > 500) {
        palette = shades();
        lastShades = now;
        if (palette.join() !== paletteKey) { paletteKey = palette.join(); mixAll(); drawn.fill(-1); }
      }
      // A day lights up every ~80 ms and fades in and out over 2.4 s.
      if (!reduce && Math.random() < 0.4) pops.push({ i: (Math.random() * cols * rows) | 0, t0: now, peak: 2 + Math.random() * 2.5 });
      glow.fill(0);
      pops = pops.filter((p) => {
        const age = (now - p.t0) / 2400;
        if (age >= 1) return false;
        glow[p.i] = Math.max(glow[p.i], p.peak * Math.sin(Math.PI * age) ** 2);
        return true;
      });
      // A diagonal wave every 9 s.
      const span = cols + rows * 0.5 + 24;
      const wave = reduce ? span * 0.6 : ((now / 9000) % 1) * span - 12;

      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const i = r * cols + c;
          const d = c + r * 0.5 - wave;
          const v = Math.min(5, base[i] + Math.exp(-(d * d) / 16) * 2.2 + glow[i]);
          const k = Math.round(v * STEPS);
          if (drawn[i] === k) continue;
          drawn[i] = k;
          const x = ox + c * P, y = r * P;
          g.clearRect(x - 2, y - 2, P, P);
          g.fillStyle = fills[k];
          g.beginPath();
          g.roundRect(x, y, S, S, R);
          g.fill();
        }
      }
    };

    const frame = (now) => {
      raf = 0;
      if (!visible) return;
      if (now - last > 32) { draw(now); last = now; }
      raf = requestAnimationFrame(frame);
    };
    const io = new IntersectionObserver(([e]) => {
      visible = e.isIntersecting;
      if (visible && !raf && !reduce) raf = requestAnimationFrame(frame);
    });
    io.observe(canvas);
    draw(performance.now());
    if (!reduce) raf = requestAnimationFrame(frame);
    return () => { cancelAnimationFrame(raf); io.disconnect(); };
  });
</script>

<div class="wall" bind:clientWidth={width} bind:clientHeight={height} aria-hidden="true">
  <canvas bind:this={canvas} style:width="{width}px" style:height="{height}px"></canvas>
</div>

<style>
  .wall { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }
  canvas { position: absolute; inset: 0; display: block; }
</style>
