// Line icons on a 24×24 grid, drawn with a round 2px stroke by Icon.svelte. Each is SVG markup;
// `FILLED` ones are solid shapes instead.

// An eight-tooth cog, like Discord's settings button.
function cog() {
  const pts = [];
  for (let i = 0; i < 8; i++) {
    const a = (i / 8) * 2 * Math.PI, w = Math.PI / 8;
    pts.push([a - w * 0.66, 7], [a - w * 0.36, 9.6], [a + w * 0.36, 9.6], [a + w * 0.66, 7]);
  }
  const d = pts.map(([a, r]) => `${(12 + r * Math.cos(a)).toFixed(2)} ${(12 + r * Math.sin(a)).toFixed(2)}`).join("L");
  return `<path d="M${d}Z"/><circle cx="12" cy="12" r="3"/>`;
}

export const ICONS = {
  upload: `<path d="M12 15V4m0 0L7.5 8.5M12 4l4.5 4.5M4 14v3a3 3 0 0 0 3 3h10a3 3 0 0 0 3-3v-3"/>`,
  play: `<path d="M8 5.6v12.8a1 1 0 0 0 1.5.86l10.6-6.4a1 1 0 0 0 0-1.72L9.5 4.74A1 1 0 0 0 8 5.6z"/>`,
  lock: `<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8.5 11V8a3.5 3.5 0 0 1 7 0v3"/>`,
  chevron: `<path d="m9 6 6 6-6 6"/>`,
  down: `<path d="m6 9 6 6 6-6"/>`,
  arrow: `<path d="M12 5v14m0 0-5-5m5 5 5-5"/>`,
  cog: cog(),
  download: `<path d="M12 4v11m0 0-4.5-4.5M12 15l4.5-4.5M5 20h14"/>`,
  mail: `<rect x="3" y="5" width="18" height="14" rx="2"/><path d="m3.5 6.5 8.5 6.5 8.5-6.5"/>`,
  check: `<path d="M5 12.5 10 17l9-10"/>`,
  chat: `<path d="M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H10l-5 4v-4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z"/>`,
  people: `<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.8-3.6 3.3-5.5 6.5-5.5s5.7 1.9 6.5 5.5M16 4.6a3.5 3.5 0 0 1 0 6.8M18.5 14.8c1.6.8 2.6 2.5 3 5.2"/>`,
  servers: `<rect x="3" y="3" width="7.5" height="7.5" rx="2.5"/><rect x="13.5" y="3" width="7.5" height="7.5" rx="3.75"/><rect x="3" y="13.5" width="7.5" height="7.5" rx="3.75"/><rect x="13.5" y="13.5" width="7.5" height="7.5" rx="2.5"/>`,
  voice: `<path d="M4 15v-3a8 8 0 0 1 16 0v3"/><path d="M4 15h3v5H5a1 1 0 0 1-1-1zM20 15h-3v5h2a1 1 0 0 0 1-1z"/>`,
  games: `<path d="M7.5 7h9a5 5 0 0 1 4.8 6.3l-.9 3.1a2.4 2.4 0 0 1-4.1.9L15 16H9l-1.3 1.3a2.4 2.4 0 0 1-4.1-.9l-.9-3.1A5 5 0 0 1 7.5 7z"/><path d="M8 10v3.5M6.25 11.75h3.5M15.5 11h.01M17.5 13h.01"/>`,
  timeline: `<path d="M5 21V4m0 0h11l-2 4 2 4H5"/>`,
  emoji: `<circle cx="12" cy="12" r="9"/><path d="M8.5 14.5a4.5 4.5 0 0 0 7 0M9 9.5h.01M15 9.5h.01"/>`,
  devices: `<rect x="2" y="4" width="14" height="10" rx="1.5"/><path d="M5 18h8M9 14v4"/><rect x="17" y="8" width="5" height="12" rx="1.2"/>`,
  phone: `<rect x="6" y="3" width="12" height="18" rx="2"/><path d="M11 18h2"/>`,
  desktop: `<rect x="3" y="4.5" width="18" height="11.5" rx="1.5"/><path d="M8 20h8M12 16v4"/>`,
  web: `<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9S9.5 5.6 12 3z"/>`,
  gift: `<rect x="3.5" y="8" width="17" height="4" rx="1"/><path d="M5 12v7a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-7M12 8v12M12 8S10.6 4 8.2 4a2 2 0 0 0 0 4M12 8s1.4-4 3.8-4a2 2 0 0 1 0 4"/>`,
  eye: `<path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>`,
  fire: `<path d="M12 3c1 4 6 5.5 6 11a6 6 0 0 1-12 0c0-3 2-4.5 2.5-7 1.2 1.2 2 2.6 2 4 .8-2.2 1.2-5.2 1.5-8z"/>`,
  star: `<path d="M12 3l2.6 5.6 6.1.7-4.5 4.2 1.2 6L12 16.6 6.6 19.5l1.2-6-4.5-4.2 6.1-.7z"/>`,
  flag: `<path d="M5 21V4m0 0h11l-2 4 2 4H5"/>`,
  tag: `<path d="M3 12V4h8l9 9-8 8z"/><path d="M7.5 8h.01"/>`,
  box: `<path d="M3 7l9-4 9 4-9 4zM3 7v10l9 4 9-4V7M12 11v10"/>`,
  door: `<path d="M5 21V4h10v17M15 4l4 2v15M11 13h.01"/>`,
  plus: `<path d="M12 5v14M5 12h14"/>`,
  gem: `<path d="M6 4h12l3 5-9 11L3 9zM3 9h18M9 4l3 16 3-16"/>`,
  avatar: `<circle cx="12" cy="8" r="4"/><path d="M4 21c1-4 4-6 8-6s7 2 8 6"/>`,
  friend: `<circle cx="10" cy="8" r="4"/><path d="M3 21c1-4 3.5-6 7-6s6 2 7 6M19 8v6M16 11h6"/>`,
  age: `<path d="M12 21v-9"/><path d="M12 12c0-4 3-7 8-7 0 5-3 8-8 7z"/><path d="M12 14c0-3-2.5-5-6.5-5 0 4 2.5 6 6.5 5z"/>`,
  stream: `<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4M10.5 8v4.5l3.5-2.25z"/>`,
  nitro: `<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/><path d="M19 15l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7z"/>`,
  coin: `<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 10h18M7 15h3"/>`,
  shield: `<path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z"/><path d="M9 12l2 2 4-4"/>`,
  plane: `<path d="M21 15.5v-1.8l-7.5-4.6V4a1.5 1.5 0 0 0-3 0v5.1L3 13.7v1.8l7.5-2.3v4.4l-2 1.5V21l3.5-1 3.5 1v-1.9l-2-1.5v-4.4z"/>`,
  leave: `<path d="M14 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-4"/><path d="M9 16l-4-4 4-4M5 12h10"/>`,
  // GitHub's mark (from Octicons, MIT licence), scaled from 16 to 24.
  github: `<path transform="scale(1.5)" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>`,
};

export const FILLED = new Set(["play", "github"]);
