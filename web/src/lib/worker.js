// Background thread: reads the zip and runs the WASM engine, so the page stays responsive.
// The page starts three of these at once, with jobs "main", "games" and "tns".
import init from "./pkg/engine.js";
import { analyze, analyzeGames, analyzeTns } from "./analyze.js";

const ready = init();

self.onmessage = async ({ data: { file, job } }) => {
  try {
    await ready;
    const run = { main: analyze, games: analyzeGames, tns: analyzeTns }[job];
    const result = await run(file, (update) => self.postMessage(update, [update.days, update.activity].filter(Boolean).map((a) => a.buffer)));
    self.postMessage({ type: "done", result });
  } catch (err) {
    self.postMessage({ type: "error", message: err?.message ?? String(err) });
  }
};
