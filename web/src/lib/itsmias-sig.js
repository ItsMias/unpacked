/* ItsMias signature — builds the ITSMIAS wordmark in the footer
   (see itsmias-sig.css). The same file runs on every site; the
   build is ported from itsmias.com's script.js so the mark is
   constructed the same way there and here. Without this file the
   signature is plain text.

   Every .mias-sig already on the page is set up when this runs.
   Pages that render their footer later (a Svelte or React app) call
   window.miasSig(el) once it is in the DOM; it returns { destroy },
   so it also works as a Svelte action. */
(function () {
  'use strict';

  // where each letter's construction line crosses it, cycled
  var LINE_Y = ['18%', '78%', '50%', '30%', '66%', '42%', '84%'];
  // stray glitch pixels around the mark: [left, top, delay]
  var PIXELS = [['-.3em', '.1em', '0ms'], ['38%', '-.3em', '220ms'], ['71%', '112%', '440ms'], ['101%', '20%', '650ms']];

  // Where the T's crossbar ends, as a % of its glyph box from the top:
  // the crossbar and stem copies are clipped at this line so they rejoin
  // exactly. Renders the glyph to a canvas, finds the run of full-width
  // ink rows at the cap top, then maps that onto the DOM box via its
  // baseline.
  var crossbarCut = function (g) {
    try {
      var cs = getComputedStyle(g), fs = parseFloat(cs.fontSize);
      var cv = document.createElement('canvas'), W = Math.ceil(fs * 1.5), H = Math.ceil(fs * 2), base = Math.round(fs * 1.4);
      cv.width = W; cv.height = H;
      var ctx = cv.getContext('2d');
      ctx.font = cs.fontWeight + ' ' + fs + 'px ' + cs.fontFamily;
      ctx.fillText(g.textContent, 0, base);
      var px = ctx.getImageData(0, 0, W, H).data, rows = [], maxW = 0;
      for (var y = 0; y < H; y++) {
        var n = 0;
        for (var x = 0; x < W; x++) if (px[(y * W + x) * 4 + 3] > 128) n++;
        rows.push(n); if (n > maxW) maxW = n;
      }
      var top = rows.findIndex(function (n) { return n > maxW * 0.6; });
      if (top < 0) return '30%';
      var bottom = top;
      while (bottom + 1 < H && rows[bottom + 1] > maxW * 0.6) bottom++;
      // baseline of the DOM glyph box, via a zero-size inline probe
      var probe = document.createElement('i');
      probe.style.cssText = 'display:inline-block;width:0;height:0;vertical-align:baseline';
      g.appendChild(probe);
      var box = g.getBoundingClientRect(), baseY = probe.getBoundingClientRect().top - box.top;
      g.removeChild(probe);
      // a hair below the crossbar so the cut never shows as a seam
      var cutY = baseY + (bottom + 1 - base) + fs * 0.02;
      return Math.max(5, Math.min(95, cutY / box.height * 100)).toFixed(2) + '%';
    } catch (e) { return '30%'; }
  };

  // never build in the fallback face: wait (briefly) for the font first
  var fontReady = document.fonts && document.fonts.load
    ? Promise.race([document.fonts.load('800 1em "ItsMias Sig"', 'ITSMA'), new Promise(function (r) { setTimeout(r, 2500); })])
        .catch(function () {})
    : Promise.resolve();

  var init = function (sig) {
    var mark = sig.querySelector('.mias-sig-mark');
    if (!mark || sig.classList.contains('is-armed')) return { destroy: function () {} };

    // split the mark into per-letter pieces; the T sits out the
    // left-to-right build and is constructed last from its strokes
    var text = mark.textContent.trim();
    var finAt = text.indexOf('T');
    mark.textContent = '';
    text.split('').forEach(function (ch, i) {
      var c = document.createElement('span');
      c.className = 'wm-ch' + (i === finAt ? ' wm-fin' : '');
      c.setAttribute('aria-hidden', 'true');
      c.style.setProperty('--i', i);
      c.style.setProperty('--ly', LINE_Y[i % LINE_Y.length]);
      // the T starts once the last regular letter is mostly built
      if (i === finAt) c.style.setProperty('--fd', ((text.length - 1) * 105 + 380) + 'ms');
      c.innerHTML = '<span class="wm-g"></span><span class="wm-s wm-s1"></span>' +
        '<span class="wm-s wm-s2"></span><span class="wm-ln"></span>';
      c.querySelectorAll('.wm-g, .wm-s').forEach(function (s) { s.textContent = ch; });
      mark.appendChild(c);
    });
    PIXELS.forEach(function (p) {
      var px = document.createElement('span');
      px.className = 'wm-px';
      px.setAttribute('aria-hidden', 'true');
      px.style.setProperty('--sx', p[0]);
      px.style.setProperty('--sy', p[1]);
      px.style.setProperty('--sd', p[2]);
      mark.appendChild(px);
    });
    sig.classList.add('is-armed');

    // Plays once the font is in and the signature is well into view;
    // resets quietly once it has fully left the screen, so it builds
    // again on the next visit to the foot of the page.
    var ready = false, inView = false, io = null;
    var play = function () { if (ready && inView) sig.classList.add('is-in'); };

    fontReady.then(function () {
      var fin = mark.querySelector('.wm-fin');
      if (fin) fin.style.setProperty('--cut', crossbarCut(fin.querySelector('.wm-g')));
      ready = true;
      play();
    });

    if ('IntersectionObserver' in window) {
      io = new IntersectionObserver(function (entries) {
        var e = entries[0];
        if (e.intersectionRatio >= 0.6) { inView = true; play(); }
        else if (!e.isIntersecting) { inView = false; sig.classList.remove('is-in'); }
      }, { threshold: [0, 0.6] });
      io.observe(sig);
    } else {
      inView = true;
      play();
    }

    return { destroy: function () { if (io) io.disconnect(); } };
  };

  window.miasSig = init;
  document.querySelectorAll('.mias-sig').forEach(init);
})();
