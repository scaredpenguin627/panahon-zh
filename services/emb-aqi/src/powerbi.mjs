/**
 * powerbi.mjs —— Power BI「发布到 Web」公开报表的最小读取器
 *
 * EMB 的空气质量页面本身不含任何数据，它们只是用 <iframe> 嵌了 Power BI 报表。
 * 所以要拿到数据，必须走 Power BI 匿名嵌入使用的三段式协议：
 *
 *   1. GET  app.powerbi.com/view?r=<base64>        → 得到该报表所在集群 (resolvedClusterUri)
 *   2. GET  {cluster}/public/reports/{key}/modelsAndExploration  → 得到 modelId + 报表结构
 *   3. POST {cluster}/public/reports/querydata     → 用语义查询换取真实行数据 (DSR 格式)
 *
 * 其中第 3 步的请求体（"配方"）是从浏览器里抓下来的固定 JSON，见 ../recipes/。
 * 没有任何依赖，只需要 Node 18+ 自带的 https。
 */

import { request as httpsRequest } from "node:https";
import { gunzipSync } from "node:zlib";
import { randomUUID } from "node:crypto";

const USER_AGENT =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36";

const POWERBI_APP = "https://app.powerbi.com";

/** 低层 HTTP：只做重试、gzip 解压和超时，保持无依赖。 */
function httpJson(url, { method = "GET", headers = {}, body = null, retries = 2, timeout = 45000 } = {}) {
  return new Promise((resolve, reject) => {
    const attempt = (left) => {
      const u = new URL(url);
      const payload = body == null ? null : Buffer.from(JSON.stringify(body));
      const req = httpsRequest(
        {
          hostname: u.hostname,
          path: u.pathname + u.search,
          method,
          headers: {
            "User-Agent": USER_AGENT,
            Accept: "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Accept-Encoding": "gzip",
            Origin: POWERBI_APP,
            Referer: `${POWERBI_APP}/`,
            ...headers,
            ...(payload ? { "Content-Type": "application/json;charset=UTF-8", "Content-Length": payload.length } : {}),
          },
        },
        (res) => {
          const chunks = [];
          res.on("data", (c) => chunks.push(c));
          res.on("end", () => {
            let buf = Buffer.concat(chunks);
            if (res.headers["content-encoding"] === "gzip") {
              try {
                buf = gunzipSync(buf);
              } catch {
                /* 某些响应声明了 gzip 但其实是明文 */
              }
            }
            const text = buf.toString("utf8");
            if (res.statusCode >= 500 && left > 0) return setTimeout(() => attempt(left - 1), 800);
            resolve({ status: res.statusCode, headers: res.headers, text });
          });
        }
      );
      req.on("error", (err) => (left > 0 ? setTimeout(() => attempt(left - 1), 800) : reject(err)));
      req.setTimeout(timeout, () => req.destroy(new Error(`timeout: ${url}`)));
      if (payload) req.write(payload);
      req.end();
    };
    attempt(retries);
  });
}

/**
 * 解析页面 iframe 里的 r= 参数。
 * 形如 {"k":"<resourceKey>","t":"<tenantId>","m":10}
 */
export function decodeEmbedParam(rParam) {
  const json = Buffer.from(decodeURIComponent(rParam), "base64").toString("utf8");
  const { k, t, m } = JSON.parse(json);
  if (!k) throw new Error("r 参数里没有 resourceKey");
  return { resourceKey: k, tenantId: t, mode: m };
}

/** 反向：用 resourceKey 拼出可嵌的 r 参数（调试用）。 */
export function encodeEmbedParam({ resourceKey, tenantId, mode = 10 }) {
  return Buffer.from(JSON.stringify({ k: resourceKey, t: tenantId, m: mode })).toString("base64");
}

// 同一集群内，浏览器前端用的是 -redirect 域名，而真正的数据接口在 -api 域名上。
const clusterCache = new Map();

/** 第 1 步：问 app.powerbi.com 这个报表落在哪个集群。 */
export async function resolveCluster(resourceKey) {
  if (clusterCache.has(resourceKey)) return clusterCache.get(resourceKey);

  const url = `${POWERBI_APP}/view?r=${encodeURIComponent(
    encodeEmbedParam({ resourceKey, tenantId: "f6f4a692-43b3-433b-92b2-65c4e6ccd920" })
  )}`;
  const { status, text } = await httpJson(url, { headers: { Accept: "text/html,*/*" } });
  if (status !== 200) throw new Error(`解析集群失败 HTTP ${status}`);

  const m = text.match(/resolvedClusterUri\s*=\s*'([^']+)'/) || text.match(/"FixedClusterUri":"([^"]+)"/);
  if (!m) throw new Error("没找到 resolvedClusterUri");

  const redirect = m[1].replace(/\/+$/, "");
  const api = redirect.replace(/-redirect\./, "-api.");
  if (api === redirect) throw new Error(`无法把集群地址转换为 API 域名: ${redirect}`);

  clusterCache.set(resourceKey, api);
  return api;
}

/** 第 2 步：拿 modelId 和报表结构（分页、视觉对象、字段投影）。 */
export async function getModelInfo(cluster, resourceKey) {
  const { status, text } = await httpJson(
    `${cluster}/public/reports/${resourceKey}/modelsAndExploration?preferReadOnlySession=true`,
    { headers: { "X-PowerBI-ResourceKey": resourceKey } }
  );
  if (status !== 200) throw new Error(`modelsAndExploration HTTP ${status}`);

  const json = JSON.parse(text);
  const report = json.exploration?.report ?? {};
  const sections = (json.exploration?.sections ?? []).map((s) => ({
    name: s.displayName || s.name,
    visuals: (s.visualContainers ?? []).map((v) => {
      let cfg = {};
      try {
        cfg = JSON.parse(v.config || "{}");
      } catch {
        /* 有的视觉对象 config 不是合法 JSON */
      }
      const sv = cfg.singleVisual ?? {};
      const projections = [];
      for (const arr of Object.values(sv.projections ?? {})) {
        for (const p of arr ?? []) projections.push(p.queryRef ?? p.nativeQueryRef ?? "?");
      }
      return { type: sv.visualType ?? "?", projections };
    }),
  }));

  return {
    modelId: report.modelId,
    reportName: report.displayName,
    reportId: report.id,
    lastUpdated: report.lastUpdatedTime,
    sections,
  };
}

/**
 * 第 3 步：把配方里的 modelId 填好，POST 出去，返回原始 dsr。
 * 三个 Header 缺一不可：X-PowerBI-ResourceKey 决定是哪个报表，
 * RequestId / ActivityId 是服务端的追踪字段，没有它们会被直接断开连接。
 */
export async function runQuery(cluster, resourceKey, modelId, recipe) {
  const body = JSON.parse(JSON.stringify(recipe));
  body.modelId = modelId;

  const { status, text } = await httpJson(`${cluster}/public/reports/querydata?synchronous=true`, {
    method: "POST",
    headers: {
      "X-PowerBI-ResourceKey": resourceKey,
      RequestId: randomUUID(),
      ActivityId: randomUUID(),
    },
    body,
  });

  if (status !== 200) throw new Error(`querydata HTTP ${status}: ${text.slice(0, 200)}`);

  const json = JSON.parse(text);
  const data = json.results?.[0]?.result?.data;
  if (!data?.dsr) {
    const err = json.results?.[0]?.result?.data?.error ?? json.error;
    throw new Error(`querydata 无数据: ${JSON.stringify(err ?? json).slice(0, 300)}`);
  }

  // descriptor（字段代号 → 字段名的对照表）是 dsr 的兄弟节点，挂进去方便解码
  const dsr = data.dsr;
  if (data.descriptor) dsr.descriptor = data.descriptor;
  return dsr;
}

/**
 * DSR（Data Shape Result）解码。
 *
 * 结构：dsr.DS[i].PH[0].DM0 = 行数组，每行 { S?, C, R? }
 *   S —— 该行的字段表（省略时沿用上一行），每项含 N(字段代号) T(类型) DN(字典名)
 *   C —— 与 S 对齐的单元格值；有 DN 的是字典下标，否则是原值
 *   R —— 重复次数（表格类视觉对象里恒为 1，可忽略）
 * 字典在 dsr.DS[i].ValueDicts[DN]。
 * 真实字段名由 descriptor.Select[].{Value,Name} 给出，如 G0 → "CAMS.Station"。
 */
export function decodeDsr(dsr) {
  const rows = [];

  for (const ds of dsr.DS ?? []) {
    const dicts = ds.ValueDicts ?? {};

    for (const phase of ds.PH ?? []) {
      for (const member of Object.values(phase)) {
        let schema = null;

        for (const item of member) {
          if (item.S) schema = item.S;
          if (!schema) continue;

          const record = {};
          schema.forEach((field, i) => {
            // 卡片类视觉对象（card）不把结果放进 C 数组，而是直接挂在字段名属性上
            const raw = item.C === undefined ? item[field.N] : item.C[i];
            if (raw === undefined || raw === null) {
              record[field.N] = null;
              return;
            }
            if (field.DN) {
              record[field.N] = dicts[field.DN]?.[raw] ?? null;
            } else if (typeof raw === "string" && /^-?\d+L$/.test(raw)) {
              record[field.N] = Number(raw.slice(0, -1)); // Power BI 用 "44L" 表示 int64
            } else if (field.T === 7 && typeof raw === "number") {
              record[field.N] = new Date(raw).toISOString(); // 时间字段是 epoch 毫秒
            } else {
              record[field.N] = raw;
            }
          });
          rows.push(record);
        }
      }
    }
  }

  // 把 G0/M0 之类的代号换成人类可读的字段名
  const alias = {};
  for (const sel of dsr.descriptor?.Select ?? []) {
    if (sel.Value && sel.Name) alias[sel.Value] = sel.Name;
  }
  return rows.map((row) => {
    const out = {};
    for (const [k, v] of Object.entries(row)) out[alias[k] ?? k] = v;
    return out;
  });
}

/** 一步到位：解析 r 参数 → 跑若干个配方 → 返回按配方名分组的行。 */
export async function queryReport(rParam, recipes) {
  const { resourceKey } = decodeEmbedParam(rParam);
  const cluster = await resolveCluster(resourceKey);
  const info = await getModelInfo(cluster, resourceKey);

  const results = {};
  for (const [name, recipe] of Object.entries(recipes)) {
    results[name] = decodeDsr(await runQuery(cluster, resourceKey, info.modelId, recipe));
  }
  return { resourceKey, cluster, info, results };
}

