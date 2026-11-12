//! DENR-EMB real-time air quality, read straight from the public Power BI report.
//!
//! The EMB pages (`ncr.emb.gov.ph/realtimeairmonitoring/`) embed a Power BI
//! "publish to web" report; the numbers are not in the page HTML. The anonymous
//! embed protocol is three requests:
//!
//! 1. `GET app.powerbi.com/view?r=…` → the cluster that hosts the report;
//! 2. `GET {cluster}/public/reports/{key}/modelsAndExploration` → `modelId`;
//! 3. `POST {cluster}/public/reports/querydata` → rows in the DSR wire format.
//!
//! The request bodies in `recipes/` were captured from the report once and are
//! replayed verbatim. If EMB restructures the report, recapture them.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::{aqi_category, AirQuality, AirQualitySource, AirQualityStation, AirQualitySummary};

/// `156a380e-…` from the iframe's `r=` parameter.
const RESOURCE_KEY: &str = "156a380e-4084-4645-8a01-6a1e845d7416";

/// The exact (already URL-encoded) `r=` value used by both EMB pages.
const EMBED_R: &str = "eyJrIjoiMTU2YTM4MGUtNDA4NC00NjQ1LThhMDEtNmExZTg0NWQ3NDE2IiwidCI6ImY2ZjRhNjkyLTQzYjMtNDMzYi05MmIyLTY1YzRlNmNjZDkyMCIsImMiOjEwfQ%3D%3D";

const POWERBI_APP: &str = "https://app.powerbi.com";
const EMB_PAGE: &str = "https://ncr.emb.gov.ph/realtimeairmonitoring/";
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36";

const RECIPE_PM10: &str = include_str!("../recipes/cams-pm10.json");
const RECIPE_PM25: &str = include_str!("../recipes/cams-pm25.json");
const RECIPE_UPDATED: &str = include_str!("../recipes/cams-updated.json");

/// Read the official NCR report and reshape it into [`AirQuality`].
pub fn fetch() -> Result<AirQuality, String> {
    let cluster = resolve_cluster()?;
    let model_id = fetch_model_id(&cluster)?;

    let pm10 = run_recipe(&cluster, &model_id, RECIPE_PM10)?;
    let pm25 = run_recipe(&cluster, &model_id, RECIPE_PM25)?;
    let updated = run_recipe(&cluster, &model_id, RECIPE_UPDATED)?;

    let mut by_station: Vec<(String, AirQualityStation)> = Vec::new();
    for row in &pm10 {
        let Some(name) = text(row, "CAMS.Station") else {
            continue;
        };
        by_station.push((
            name.to_owned(),
            AirQualityStation {
                name_zh: station_zh(name).to_owned(),
                name: Some(name.to_owned()),
                pm10: number(row, "Sum(CAMS.PM10AQI)").map(|value| value as f32),
                pm25: None,
                so2: None,
                no2: None,
                co: None,
                o3: None,
                aqi: None,
                pollutant: None,
                category: String::new(),
            },
        ));
    }
    for row in &pm25 {
        let Some(name) = text(row, "CAMS.Station") else {
            continue;
        };
        let pm25_value = number(row, "Sum(CAMS.PM25AQI)").map(|value| value as f32);
        match by_station.iter_mut().find(|(key, _)| key == name) {
            Some((_, station)) => station.pm25 = pm25_value,
            None => by_station.push((
                name.to_owned(),
                AirQualityStation {
                    name_zh: station_zh(name).to_owned(),
                    name: Some(name.to_owned()),
                    pm10: None,
                    pm25: pm25_value,
                    so2: None,
                    no2: None,
                    co: None,
                    o3: None,
                    aqi: None,
                    pollutant: None,
                    category: String::new(),
                },
            )),
        }
    }

    let mut stations: Vec<AirQualityStation> = by_station
        .into_iter()
        .map(|(_, mut station)| {
            let worst = [("PM10", station.pm10), ("PM2.5", station.pm25)]
                .into_iter()
                .filter_map(|(key, value)| value.map(|value| (key, value)))
                .max_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((pollutant, value)) = worst {
                station.aqi = Some(value.round().clamp(0.0, 500.0) as u16);
                station.pollutant = Some(pollutant.to_owned());
            }
            station.category = station
                .aqi
                .map(|aqi| aqi_category(aqi).to_owned())
                .unwrap_or_else(|| "暂无".into());
            station
        })
        .collect();
    stations.sort_by(|a, b| b.aqi.unwrap_or(0).cmp(&a.aqi.unwrap_or(0)));

    if stations.is_empty() {
        return Err("EMB 报表返回了 0 个监测站".into());
    }

    let values: Vec<u16> = stations.iter().filter_map(|station| station.aqi).collect();
    let avg = if values.is_empty() {
        None
    } else {
        Some(
            (values.iter().map(|value| u32::from(*value)).sum::<u32>() / values.len() as u32)
                as u16,
        )
    };
    let max = stations
        .iter()
        .max_by_key(|station| station.aqi.unwrap_or(0))
        .filter(|station| station.aqi.is_some());
    let data_clock = updated
        .iter()
        .find_map(|row| number(row, "Min(CAMS.Date_Time)"))
        .map(|ms| iso_from_ms(ms as i64));

    Ok(AirQuality {
        source: AirQualitySource {
            provider: "DENR-EMB（菲律宾环境管理局）".into(),
            report: Some("AQI_new".into()),
            report_id: None,
            page: EMB_PAGE.into(),
            note: "直接读取 EMB 公开发布的 Power BI 报表；数值与官方页面一致。".into(),
        },
        quality: crate::DataQuality::Official,
        updated_at: data_clock.clone(),
        data_clock: data_clock.as_deref().map(|iso| {
            // "2026-11-08T06:00:00Z" → "2026-11-08 06:00"
            iso.replace('T', " ")
                .replace('Z', "")
                .chars()
                .take(16)
                .collect()
        }),
        fetched_at: Some(iso_from_ms(now_ms())),
        summary: AirQualitySummary {
            station_count: stations.len() as u16,
            avg_aqi: avg,
            max_aqi: max.and_then(|station| station.aqi),
            max_station: max.map(|station| station.name_zh.clone()),
            category: max.map(|station| station.category.clone()),
        },
        stations,
        trend: Vec::new(),
    })
}

fn resolve_cluster() -> Result<String, String> {
    let html = get_text(&format!("{POWERBI_APP}/view?r={EMBED_R}"))?;
    let marker = "resolvedClusterUri = '";
    let start = html
        .find(marker)
        .map(|index| index + marker.len())
        .ok_or_else(|| "Power BI 页面没有返回集群地址".to_owned())?;
    let redirect = html[start..]
        .split('\'')
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    let api = redirect.replace("-redirect.", "-api.");
    if api == redirect {
        return Err(format!("无法把集群地址转换成 API 域名：{redirect}"));
    }
    Ok(api)
}

fn fetch_model_id(cluster: &str) -> Result<u64, String> {
    let url = format!(
        "{cluster}/public/reports/{RESOURCE_KEY}/modelsAndExploration?preferReadOnlySession=true"
    );
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(25))
        .set("User-Agent", BROWSER_UA)
        .set("X-PowerBI-ResourceKey", RESOURCE_KEY)
        .set("Accept", "application/json")
        .call()
        .map_err(|err| format!("读取报表模型失败：{err}"))?;
    let value: Value = response
        .into_json()
        .map_err(|err| format!("报表模型无法解析：{err}"))?;
    value
        .pointer("/exploration/report/modelId")
        .and_then(Value::as_u64)
        .ok_or_else(|| "报表模型里没有 modelId".to_owned())
}

fn run_recipe(
    cluster: &str,
    model_id: &u64,
    recipe: &str,
) -> Result<Vec<Map<String, Value>>, String> {
    let mut body: Value =
        serde_json::from_str(recipe).map_err(|err| format!("查询配方无法解析：{err}"))?;
    body["modelId"] = Value::from(*model_id);

    let response = ureq::post(&format!(
        "{cluster}/public/reports/querydata?synchronous=true"
    ))
    .timeout(std::time::Duration::from_secs(25))
    .set("User-Agent", BROWSER_UA)
    .set("X-PowerBI-ResourceKey", RESOURCE_KEY)
    .set("RequestId", &uuid_like())
    .set("ActivityId", &uuid_like())
    .set("Content-Type", "application/json;charset=UTF-8")
    .set("Accept", "application/json, text/plain, */*")
    .send_string(&body.to_string())
    .map_err(|err| format!("查询报表数据失败：{err}"))?;

    let value: Value = response
        .into_json()
        .map_err(|err| format!("报表数据无法解析：{err}"))?;

    let data = value
        .pointer("/results/0/result/data")
        .ok_or_else(|| "报表响应里没有数据节点".to_owned())?;
    let dsr = data
        .get("dsr")
        .ok_or_else(|| "报表响应里没有 dsr".to_owned())?;
    Ok(decode_dsr(dsr, data.get("descriptor")))
}

/// Decode the compressed DSR row format into plain rows keyed by select name.
fn decode_dsr(dsr: &Value, descriptor: Option<&Value>) -> Vec<Map<String, Value>> {
    let mut alias: Vec<(String, String)> = Vec::new();
    if let Some(selects) = descriptor
        .and_then(|value| value.get("Select"))
        .and_then(Value::as_array)
    {
        for select in selects {
            if let (Some(value), Some(name)) = (
                select.get("Value").and_then(Value::as_str),
                select.get("Name").and_then(Value::as_str),
            ) {
                alias.push((value.to_owned(), name.to_owned()));
            }
        }
    }

    let mut decoded: Vec<Map<String, Value>> = Vec::new();
    let Some(data_sets) = dsr.get("DS").and_then(Value::as_array) else {
        return decoded;
    };

    for data_set in data_sets {
        let dicts = data_set.get("ValueDicts");
        let Some(phases) = data_set.get("PH").and_then(Value::as_array) else {
            continue;
        };
        for phase in phases {
            let Some(members) = phase.as_object() else {
                continue;
            };
            for member in members.values() {
                let Some(items) = member.as_array() else {
                    continue;
                };
                let mut schema: Vec<Value> = Vec::new();
                for item in items {
                    if let Some(fields) = item.get("S").and_then(Value::as_array) {
                        schema = fields.clone();
                    }
                    if schema.is_empty() {
                        continue;
                    }
                    let mut row = Map::new();
                    for (index, field) in schema.iter().enumerate() {
                        let Some(name) = field.get("N").and_then(Value::as_str) else {
                            continue;
                        };
                        let raw = match item.get("C") {
                            Some(cells) => cells.get(index),
                            None => item.get(name),
                        };
                        let Some(raw) = raw else { continue };
                        if raw.is_null() {
                            continue;
                        }
                        let value = match field.get("DN").and_then(Value::as_str) {
                            Some(dict_name) => dicts
                                .and_then(|dicts| dicts.get(dict_name))
                                .and_then(|dict| dict.get(raw.as_u64().unwrap_or(0) as usize))
                                .cloned()
                                .unwrap_or(Value::Null),
                            None => {
                                // Power BI serialises int64 as "44L".
                                match raw.as_str() {
                                    Some(text) if text.ends_with('L') => text[..text.len() - 1]
                                        .parse::<i64>()
                                        .map(Value::from)
                                        .unwrap_or_else(|_| raw.clone()),
                                    _ => raw.clone(),
                                }
                            }
                        };
                        row.insert(name.to_owned(), value);
                    }
                    decoded.push(row);
                }
            }
        }
    }

    if alias.is_empty() {
        return decoded;
    }
    decoded
        .into_iter()
        .map(|row| {
            let mut out = Map::new();
            for (key, value) in row {
                let name = alias
                    .iter()
                    .find(|(code, _)| *code == key)
                    .map(|(_, name)| name.clone())
                    .unwrap_or(key);
                out.insert(name, value);
            }
            out
        })
        .collect()
}

fn get_text(url: &str) -> Result<String, String> {
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(25))
        .set("User-Agent", BROWSER_UA)
        .set("Accept", "text/html,application/xhtml+xml,*/*")
        .call()
        .map_err(|err| format!("请求 {url} 失败：{err}"))?
        .into_string()
        .map_err(|err| format!("{url} 返回内容无法读取：{err}"))
}

fn text<'a>(row: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn number(row: &Map<String, Value>, key: &str) -> Option<f64> {
    let value = row.get(key)?;
    value.as_f64().or_else(|| {
        value
            .as_str()
            .and_then(|text| text.trim_end_matches('L').parse().ok())
    })
}

/// Chinese names for the 16 NCR monitoring stations.
fn station_zh(name: &str) -> &str {
    match name {
        "Caloocan (Univ. of East)" => "卡洛奥坎（东方大学）",
        "Makati" => "马卡蒂",
        "Mandaluyong" => "曼达卢永",
        "Manila" => "马尼拉",
        "Marikina (Parang)" => "马里基纳（帕朗）",
        "Marikina CEMO" => "马里基纳 CEMO",
        "Muntinlupa (Bilibid)" => "蒙廷卢帕（比利比德）",
        "Muntinlupa (Filinvest City)" => "蒙廷卢帕（费林维斯特城）",
        "Navotas" => "纳沃塔斯",
        "Parañaque (Don Bosco)" => "帕拉尼亚克（唐博斯科）",
        "Parañaque (PEATC)" => "帕拉尼亚克（PEATC）",
        "Pasay" => "帕赛",
        "Quezon City (Ateneo)" => "奎松市（雅典耀）",
        "San Juan City" => "圣胡安市",
        "Taguig (TUP)" => "达义（菲律宾理工大学）",
        "Valenzuela" => "巴伦苏埃拉",
        other => other,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// A UUID-shaped request identifier. The endpoint only needs it to be unique.
fn uuid_like() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = now_ms() as u64 ^ ((now_ms() as u64) << 17);
    let tick = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mix = nanos.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17)
        ^ tick.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    format!(
        "{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}",
        (mix >> 32) as u32,
        (mix >> 16) as u16,
        (mix & 0xFFF) as u16,
        ((mix >> 12) & 0xFFF) as u16,
        (mix.rotate_left(8) & 0xFFFF_FFFF_FFFF) as u64,
    )
}

/// Format epoch milliseconds as `YYYY-MM-DDTHH:MM:SSZ` without pulling in chrono.
fn iso_from_ms(ms: i64) -> String {
    let seconds = ms.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let remainder = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        remainder / 3600,
        (remainder % 3600) / 60,
        remainder % 60
    )
}

/// Howard Hinnant's days-to-civil-date algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_round_trips_through_civil_dates() {
        assert_eq!(iso_from_ms(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_ms(1_700_000_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(iso_from_ms(1_794_074_400_000), "2026-11-07T18:00:00Z");
    }

    #[test]
    fn uuids_have_the_expected_shape() {
        let id = uuid_like();
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|ch| *ch == '-').count(), 4);
    }

    #[test]
    fn dsr_rows_are_decoded_and_aliased() {
        let dsr = serde_json::json!({
            "DS": [{
                "PH": [{ "DM0": [
                    { "S": [{ "N": "G0", "T": 1, "DN": "D0" }, { "N": "M0", "T": 4 }], "C": [0, 44] },
                    { "C": [1, 36] }
                ]}],
                "ValueDicts": { "D0": ["Caloocan", "Makati"] }
            }]
        });
        let descriptor = serde_json::json!({
            "Select": [
                { "Value": "G0", "Name": "CAMS.Station" },
                { "Value": "M0", "Name": "Sum(CAMS.PM10AQI)" }
            ]
        });
        let rows = decode_dsr(&dsr, Some(&descriptor));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["CAMS.Station"], "Caloocan");
        assert_eq!(rows[0]["Sum(CAMS.PM10AQI)"], 44.0);
        assert_eq!(rows[1]["CAMS.Station"], "Makati");
    }
}
