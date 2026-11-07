/**
 * collect.mjs —— 业务层：把 Power BI 的三张表拼成 App 能直接用的站点数组
 *
 *   recipes/cams-pm10.json    → CAMS.Station / Sum(CAMS.PM10AQI) / CAMS.PM10REMARKS / CAMS.Address
 *   recipes/cams-pm25.json    → CAMS.Station / Sum(CAMS.PM25AQI) / CAMS.PM25REMARKS / CAMS.Address
 *   recipes/cams-updated.json → Min(CAMS.Date_Time)
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { queryReport, encodeEmbedParam } from "./powerbi.mjs";
import { STATIONS_ZH, REMARK_ZH, levelOf } from "./labels.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const RECIPE_DIR = join(HERE, "..", "recipes");

const load = (name) => JSON.parse(readFileSync(join(RECIPE_DIR, `${name}.json`), "utf8"));

export const RECIPES = {
  pm10: load("cams-pm10"),
  pm25: load("cams-pm25"),
  updated: load("cams-updated"),
};

/** EMB NCR 实时空气质量报表（ncr.emb.gov.ph/realtimeairmonitoring/ 里的主报表） */
export const EMB_NCR_REALTIME = {
  resourceKey: "156a380e-4084-4645-8a01-6a1e845d7416",
  tenantId: "f6f4a692-43b3-433b-92b2-65c4e6ccd920",
  label: "EMB-NCR 实时空气质量",
};

const num = (v) => (typeof v === "number" ? v : v == null ? null : Number.isFinite(Number(v)) ? Number(v) : null);
const slug = (s) =>
  s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

/**
 * 抓取并整理一个报表。
 * @param {string} resourceKey Power BI 报表的 resourceKey（iframe r= 参数里的 k）
 */
export async function collect(resourceKey = EMB_NCR_REALTIME.resourceKey) {
  const rParam = encodeEmbedParam({ resourceKey, tenantId: EMB_NCR_REALTIME.tenantId });
  const { results, info } = await queryReport(rParam, RECIPES);

  // PM10 与 PM2.5 的列顺序不同，这里统一按"字段名"取值，不依赖位置
  const byStation = new Map();

  for (const row of results.pm10) {
    const name = row["CAMS.Station"];
    if (!name) continue;
    byStation.set(name, {
      name,
      address: row["CAMS.Address"] ?? null,
      pm10: num(row["Sum(CAMS.PM10AQI)"]),
      pm10Remark: row["CAMS.PM10REMARKS"] ?? null,
    });
  }

  for (const row of results.pm25) {
    const name = row["CAMS.Station"];
    if (!name) continue;
    const prev = byStation.get(name) ?? { name, address: null };
    byStation.set(name, {
      ...prev,
      address: prev.address ?? row["CAMS.Address"] ?? null,
      pm25: num(row["Sum(CAMS.PM25AQI)"]),
      pm25Remark: row["CAMS.PM25REMARKS"] ?? null,
    });
  }

  const stations = [...byStation.values()]
    .map((s) => {
      // 综合 AQI 取两项中较差的那个，并记录首要污染物
      const candidates = [
        { key: "PM10", value: s.pm10 },
        { key: "PM2.5", value: s.pm25 },
      ].filter((c) => c.value != null);
      const worst = candidates.sort((a, b) => b.value - a.value)[0] ?? null;
      const aqi = worst?.value ?? null;
      const level = levelOf(aqi);
      // 评级跟着首要污染物走，避免出现"PM2.5 55 却标 Good"这类自相矛盾
      const remark =
        (worst?.key === "PM2.5" ? s.pm25Remark : s.pm10Remark) ?? s.pm10Remark ?? s.pm25Remark ?? null;

      return {
        id: slug(s.name),
        name: s.name,
        nameZh: STATIONS_ZH[s.name] ?? s.name,
        address: s.address,
        pm10: s.pm10,
        pm25: s.pm25,
        aqi,
        pollutant: worst?.key ?? null,
        category: level.zh,
        categoryEn: level.en,
        color: level.color,
        remark, // EMB 原始评级（多数站点为空）
        remarkZh: remark ? (REMARK_ZH[remark] ?? remark) : null,
      };
    })
    .sort((a, b) => (b.aqi ?? -1) - (a.aqi ?? -1));

  const updatedRaw = results.updated.find((r) => r["Min(CAMS.Date_Time)"])?.["Min(CAMS.Date_Time)"] ?? null;
  const worst = stations.find((s) => s.aqi != null) ?? null;
  const values = stations.map((s) => s.aqi).filter((v) => v != null);

  return {
    source: {
      provider: "DENR-EMB（菲律宾环境管理局）",
      report: info.reportName,
      reportId: info.reportId,
      page: "https://ncr.emb.gov.ph/realtimeairmonitoring/",
      note: "数据由 EMB 发布在 Power BI 公开报表中，本项目按报表原始口径转述。",
    },
    updatedAt: updatedRaw, // 监测数据时间（不是抓取时间）
    // 报表里的时钟读数。EMB 的 Date_Time 存在时区歧义（存的是当地钟点，却按 UTC 传输），
    // 这里额外给出"报表上的钟点"字符串，App 想显示"数据时间 18:00"时直接用它。
    dataClock: updatedRaw ? new Date(updatedRaw).toISOString().slice(0, 16).replace("T", " ") : null,
    fetchedAt: new Date().toISOString(),
    summary: {
      stationCount: stations.length,
      avgAqi: values.length ? Math.round(values.reduce((a, b) => a + b, 0) / values.length) : null,
      maxAqi: worst?.aqi ?? null,
      maxStation: worst?.nameZh ?? null,
      category: worst ? worst.category : null,
    },
    stations,
  };
}

// 直接运行时打印一份样例，方便先看效果
if (process.argv[1] && process.argv[1].endsWith("collect.mjs")) {
  const data = await collect(process.argv[2]);
  console.log(JSON.stringify(data, null, 2));
}

