// The demo account's numbers that the home page previews show, worked out with the same functions
// as the real pages.
import {
  activity, allTags, badges, buildView, deviceStats, gameStats, level, messageStats, overview,
  peopleStats, purchaseStats, serverStats, tagHistory, timeline, voiceStats, yearStats,
} from "./stats.js";

const DAY = 86_400_000;

export function summarize({ result, games }) {
  const facts = result.facts;
  const view = buildView(facts, facts.time_zone);
  const o = overview(view, facts, "all");
  const packageDate = result.sources.packageDate;

  // One calendar year for the mini heatmap, Monday first.
  const year = 2024;
  const first = Date.UTC(year, 0, 1);
  const lead = (new Date(first).getUTCDay() + 6) % 7;
  const cells = Array.from({ length: lead }, () => -1);
  for (let t = first; new Date(t).getUTCFullYear() === year; t += DAY) {
    cells.push(level(view.daily.get(new Date(t).toISOString().slice(0, 10)) ?? 0, view.breaks));
  }

  const items = timeline(view, facts, games, result);
  const voice = voiceStats(view, facts, "all");
  const voiceYears = new Map();
  for (const [d, h] of facts.voice.days) {
    const y = new Date(d * DAY).getUTCFullYear();
    voiceYears.set(y, (voiceYears.get(y) ?? 0) + h);
  }

  return {
    facts,
    view,
    profile: facts.profile,
    badges: badges(facts.profile, packageDate),
    tag: tagHistory(allTags(facts, games)).at(-1),
    o,
    clock: Array.from({ length: 24 }, (_, h) => o.hours.reduce((s, row) => s + row[h], 0)),
    year,
    cells,
    best: yearStats(view, year).best,
    longest: facts.voice.longest[0],
    months: activity(view, facts, "all", packageDate).points,
    words: messageStats(view, facts, "all").words_top.slice(0, 8),
    messages: messageStats(view, facts, "all"),
    people: peopleStats(view, facts, "all").ranked.slice(0, 3),
    servers: serverStats(view, facts, "all", result.sources).ranked.slice(0, 3),
    voice,
    voiceYears: [...voiceYears].sort((a, b) => a[0] - b[0]),
    games: gameStats(view, facts, games, "all").list.slice(0, 4),
    timeline: [items[0], items.filter((x) => x.icon === "flag").at(-1), items.filter((x) => x.icon === "tag").at(-1)].filter(Boolean),
    emoji: facts.emojis.top.slice(0, 6),
    devices: deviceStats(view, facts, "all").clients.slice(0, 3),
    money: purchaseStats(facts, "all"),
    ads: facts.ad_profile,
  };
}
