/**
 * capture-recipe.mjs —— 给新报表抓查询配方
 *
 * 用途：EMB 换了报表、或你想接另外几个嵌入（比如气体的 AQI_new_GAS），
 * 用无头 Chrome 打开报表，把它真实发出的 querydata 请求体存下来，
 * 之后就能像 recipes/cams-pm10.json 一样被 powerbi.mjs 回放。
 *
 * 用法：
 *   node tools/capture-recipe.mjs <resourceKey> [输出目录]
 *   node tools/capture-recipe.mjs 156a380e-4084-4645-8a01-6a1e845d7416
 *
 * 依赖：本机已安装 Chrome。除了 node 自带模块，不需要装别的包。
 */

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const CHROME_CANDIDATES = [
  "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
];

function findChrome() {
  const hit = CHROME_CANDIDATES.find((p) => fs.existsSync(p));
  if (!hit) throw new Error("没找到 Chrome，请改一下 CHROME_CANDIDATES 里的路径");
  return hit;
}

const resourceKey = process.argv[2];
if (!resourceKey) {
  console.error("用法: node tools/capture-recipe.mjs <resourceKey> [输出目录]");
  process.exit(1);
}

const outDir = path.resolve(process.argv[3] ?? "./captured");
fs.mkdirSync(outDir, { recursive: true });

const PORT = 9333 + Math.floor(Math.random() * 200);
const EMBED = `https://app.powerbi.com/view?r=${encodeURIComponent(
  Buffer.from(JSON.stringify({ k: resourceKey, t: "f6f4a692-43b3-433b-92b2-65c4e6ccd920", m: 10 })).toString("base64")
)}`;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const chrome = spawn(
  findChrome(),
  [
    "--headless=new",
    `--remote-debugging-port=${PORT}`,
    `--user-data-dir=${path.join(outDir, ".chrome-profile")}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-gpu",
    "--window-size=1600,1000",
    "about:blank",
  ],
  { detached: true, stdio: "ignore" }
);
chrome.unref();

let wsUrl = null;
for (let i = 0; i < 60; i++) {
  try {
    const r = await fetch(`http://127.0.0.1:${PORT}/json/version`);
    wsUrl = (await r.json()).webSocketDebuggerUrl;
    if (wsUrl) break;
  } catch {
    /* 还没起来 */
  }
  await sleep(250);
}
if (!wsUrl) throw new Error("Chrome DevTools 没起来");

const ws = new WebSocket(wsUrl);
await new Promise((res, rej) => {
  ws.onopen = res;
  ws.onerror = rej;
});

let msgId = 0;
const pending = new Map();
const hits = [];

ws.onmessage = (ev) => {
  const m = JSON.parse(ev.data);
  if (m.id && pending.has(m.id)) {
    const { resolve, reject } = pending.get(m.id);
    pending.delete(m.id);
    m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result);
    return;
  }
  if (m.method === "Network.requestWillBeSent" && /querydata/i.test(m.params.request.url) && m.params.request.postData) {
    hits.push({ requestId: m.params.requestId, sessionId: m.sessionId, postData: m.params.request.postData });
    console.log(`捕获 querydata #${hits.length}`);
  }
};

const send = (method, params = {}, sessionId) =>
  new Promise((resolve, reject) => {
    const id = ++msgId;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  });

const { targetId } = await send("Target.createTarget", { url: "about:blank" });
const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
await send("Network.enable", { maxResourceBufferSize: 200 * 1024 * 1024 }, sessionId);
await send("Page.enable", {}, sessionId);
await send("Page.navigate", { url: EMBED }, sessionId);

// 报表首屏大约要 15-25 秒；这里等请求数稳定下来
let last = -1;
for (let i = 0; i < 40; i++) {
  await sleep(1000);
  if (hits.length > 0 && hits.length === last) break;
  last = hits.length;
}
await sleep(2000);

function selectsOf(body) {
  const names = [];
  for (const q of body.queries ?? [])
    for (const c of q.Query?.Commands ?? [])
      for (const s of c.SemanticQueryDataShapeCommand?.Query?.Select ?? []) names.push(s.Name);
  return names;
}

let n = 0;
for (const h of hits) {
  const body = JSON.parse(h.postData);
  const selects = selectsOf(body);
  const file = path.join(outDir, `query-${String(++n).padStart(2, "0")}.json`);
  body.modelId = null; // 回放时按 report 现取
  fs.writeFileSync(file, JSON.stringify(body));
  console.log(`  ${path.basename(file)}  [${selects.join(", ")}]`);
}

console.log(`\n共 ${hits.length} 个查询，已写入 ${outDir}`);
console.log("挑出你要的那几个，改成 recipes/ 下的文件名即可（记得把 modelId 置为 null）。");

ws.close();
chrome.kill();
await sleep(300);
process.exit(0);

