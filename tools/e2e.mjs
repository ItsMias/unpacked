// End-to-end check in headless Chromium: loads a real package through the page's file input,
// waits for the Overview, screenshots it and lists every network request and console error.
// Usage: node tools/e2e.mjs <chrome-headless-shell> <package.zip> <out-dir> [url] [width]
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const [chrome, zip, out, url = "http://localhost:5173/", width = "1440"] = process.argv.slice(2);
mkdirSync(out, { recursive: true });

const proc = spawn(chrome, ["--no-sandbox", "--hide-scrollbars", "--remote-debugging-port=9334", "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try { target = (await (await fetch("http://127.0.0.1:9334/json")).json()).find((t) => t.type === "page"); } catch {}
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0;
const pending = new Map();
const requests = [];
const logs = [];
ws.onmessage = ({ data }) => {
  const m = JSON.parse(data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
  if (m.method === "Network.requestWillBeSent") requests.push(m.params.request.url);
  if (m.method === "Runtime.consoleAPICalled" && m.params.type !== "debug") logs.push(m.params.args.map((a) => a.value ?? a.description).join(" "));
  if (m.method === "Runtime.exceptionThrown") logs.push("EXCEPTION " + m.params.exceptionDetails.exception?.description);
};
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evaluate = async (expr) => (await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const shot = async (name) => {
  const { result } = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
  writeFileSync(`${out}/${name}.png`, Buffer.from(result.data, "base64"));
};

await send("Page.enable");
await send("Runtime.enable");
await send("Network.enable");
await send("Emulation.setDeviceMetricsOverride", { width: +width, height: 900, deviceScaleFactor: 1, mobile: +width < 700 });
await send("Page.navigate", { url });
await sleep(1500);

const { root } = (await send("DOM.getDocument")).result;
const { nodeId } = (await send("DOM.querySelector", { nodeId: root.nodeId, selector: "input[type=file]" })).result;
const t0 = Date.now();
await send("DOM.setFileInputFiles", { nodeId, files: [resolve(zip)] });

let lastStage = "";
const loadingShots = [3, 9, 18];
for (;;) {
  await sleep(500);
  if (loadingShots.length && Date.now() - t0 > loadingShots[0] * 1000) await shot(`loading-${loadingShots.shift()}s-${width}`);
  if (await evaluate(`!!document.querySelector(".profile, .grid")`)) break;
  const err = await evaluate(`document.querySelector(".error")?.textContent`);
  if (err) { console.log("ERROR:", err); break; }
  const stage = await evaluate(`document.querySelector(".steps li.active strong")?.textContent`);
  if (stage && stage !== lastStage) { console.log(`${((Date.now() - t0) / 1000).toFixed(1)}s  ${stage}`); lastStage = stage; }
  if (Date.now() - t0 > 900_000) { console.log("TIMEOUT"); break; }
}
console.log(`page ready in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
// Game history keeps loading in the background; the top bar shows a status until it's done.
for (let i = 0; i < 1200 && (await evaluate(`!!document.querySelector(".status")`)); i++) await sleep(500);
console.log(`game history done in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
await sleep(2500); // avatars
const h = await evaluate(`document.documentElement.scrollHeight`);
await send("Emulation.setDeviceMetricsOverride", { width: +width, height: h, deviceScaleFactor: 1, mobile: +width < 700 });
await sleep(500);
await shot(`overview-${width}`);
console.log("headline:", await evaluate(`[...document.querySelectorAll(".headline div")].map((d) => d.innerText.replace(/\\n/g, ": ")).join(" | ")`));
console.log("sources:", await evaluate(`[...document.querySelectorAll(".src > div")].map((d) => d.innerText.replace(/\\n/g, " ")).join(" | ")`));

const hosts = [...new Set(requests.map((u) => { try { return new URL(u).origin; } catch { return u.slice(0, 30); } }))];
console.log("request origins:", hosts);
console.log("console:", logs.filter((l) => !String(l).startsWith("[vite")).slice(0, 20));
ws.close();
proc.kill();
