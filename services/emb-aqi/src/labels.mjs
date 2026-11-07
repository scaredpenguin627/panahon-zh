/**
 * labels.mjs —— 中文化映射：站点名、EMB 原始评级、AQI 分级色带
 *
 * 站点中文名采用菲律宾地名通用译名，可按自己的偏好改。
 * 分级沿用 EPA/PH 通用的 0-500 AQI 六段划分，与 EMB 报表上的 Good/Fair 口径一致。
 */

/** 16 个 NCR 监测站的中文名（key 与报表里的 CAMS.Station 完全一致） */
export const STATIONS_ZH = {
  "Caloocan (Univ. of East)": "卡洛奥坎（东方大学）",
  Makati: "马卡蒂",
  Mandaluyong: "曼达卢永",
  Manila: "马尼拉",
  "Marikina (Parang)": "马里基纳（帕朗）",
  "Marikina CEMO": "马里基纳 CEMO",
  "Muntinlupa (Bilibid)": "蒙廷卢帕（比利比德）",
  "Muntinlupa (Filinvest City)": "蒙廷卢帕（费林维斯特城）",
  Navotas: "纳沃塔斯",
  "Parañaque (Don Bosco)": "帕拉尼亚克（唐博斯科）",
  "Parañaque (PEATC)": "帕拉尼亚克（PEATC）",
  Pasay: "帕赛",
  "Quezon City (Ateneo)": "奎松市（雅典耀）",
  "San Juan City": "圣胡安市",
  "Taguig (TUP)": "达义（菲律宾理工大学）",
  Valenzuela: "巴伦苏埃拉",
};

/** EMB 报表里的 REMARKS 原文 → 中文 */
export const REMARK_ZH = {
  Good: "优",
  Fair: "良",
  Moderate: "轻度污染",
  Poor: "中度污染",
  "Very Poor": "重度污染",
  Unhealthy: "中度污染",
  "Very Unhealthy": "重度污染",
  Hazardous: "严重污染",
};

/** AQI 六段分级（含 UI 色值） */
export const AQI_LEVELS = [
  { max: 50, en: "Good", zh: "优", color: "#00B050" },
  { max: 100, en: "Moderate", zh: "良", color: "#E8C400" },
  { max: 150, en: "Unhealthy for Sensitive Groups", zh: "轻度污染", color: "#F07C00" },
  { max: 200, en: "Unhealthy", zh: "中度污染", color: "#E4002B" },
  { max: 300, en: "Very Unhealthy", zh: "重度污染", color: "#8B2E8B" },
  { max: 500, en: "Hazardous", zh: "严重污染", color: "#7A1B1B" },
];

/** 按数值取分级；数值缺失时返回"暂无数据"。 */
export function levelOf(aqi) {
  if (aqi == null || Number.isNaN(aqi)) {
    return { en: "No Data", zh: "暂无数据", color: "#9AA5B1" };
  }
  for (const lv of AQI_LEVELS) if (aqi <= lv.max) return { en: lv.en, zh: lv.zh, color: lv.color };
  const last = AQI_LEVELS[AQI_LEVELS.length - 1];
  return { en: last.en, zh: last.zh, color: last.color };
}
