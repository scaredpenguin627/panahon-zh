/**
 * server.mjs —— 给 App 用的最小 HTTP 服务
 *
 *   GET /api/aqi            全部站点（默认缓存 5 分钟）
 *   GET /api/aqi?refresh=1  强制刷新
 *   GET /health             存活检查
 *
 * 启动：node server.mjs            （默认 8787 端口）
 * 部署到 Vercel / Cloudflare 时，把 collect() 的返回值塞进各自的 handler 即可。
 */

import { createServer } from "node:http";
import { collect } from "./src/collect.mjs";

const PORT = Number(process.env.PORT ?? 8787);
const TTL_MS = Number(process.env.CACHE_TTL_MS ?? 5 * 60 * 1000);

let cache = { at: 0, data: null };
let inflight = null;

/** 单飞 + 缓存：并发请求只打一次上游，失败时回退到上一次成功的数据。 */
async function getData(force = false) {
  const fresh = Date.now() - cache.at < TTL_MS;
  if (!force && fresh && cache.data) return { data: cache.data, cached: true };

  if (!inflight) {
    inflight = collect()
      .then((data) => {
        cache = { at: Date.now(), data };
        return data;
      })
      .catch((err) => {
        if (cache.data) return cache.data; // 上游抖动时用旧数据兜底
        throw err;
      })
      .finally(() => {
        inflight = null;
      });
  }

  return { data: await inflight, cached: false };
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", `http://${req.headers.host ?? "localhost"}`);

  // 手机 App 直连时要跨域，这里放开
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type, If-None-Match");
  if (req.method === "OPTIONS") {
    res.writeHead(204).end();
    return;
  }

  if (url.pathname === "/health") {
    res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
    res.end(JSON.stringify({ ok: true, cachedAt: cache.at ? new Date(cache.at).toISOString() : null }));
    return;
  }

  if (url.pathname === "/api/aqi") {
    try {
      const { data, cached } = await getData(url.searchParams.get("refresh") === "1");
      const body = JSON.stringify({ ...data, _cached: cached });
      res.writeHead(200, {
        "Content-Type": "application/json; charset=utf-8",
        "Cache-Control": `public, max-age=${Math.floor(TTL_MS / 1000)}`,
      });
      res.end(body);
    } catch (err) {
      res.writeHead(502, { "Content-Type": "application/json; charset=utf-8" });
      res.end(JSON.stringify({ error: "upstream_failed", message: String(err?.message ?? err) }));
    }
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json; charset=utf-8" });
  res.end(JSON.stringify({ error: "not_found", paths: ["/api/aqi", "/health"] }));
});

server.listen(PORT, () => {
  console.log(`AQI 服务已启动: http://127.0.0.1:${PORT}/api/aqi`);
});
