//! Stable data boundary for the clean-room PANaHON rebuild.
//! The app is usable offline and treats the public API as an optional adapter.

use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::time::Duration;

pub const API_ROOT: &str = "https://api.mobile.panahon.gov.ph/api/v1";
pub const OFFICIAL_MAP_URL: &str = "https://panahon.gov.ph/?embed=panahon-mobile";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub region: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrentWeather {
    pub temperature_c: f32,
    pub feels_like_c: f32,
    pub humidity_percent: u8,
    pub wind_kph: f32,
    pub wind_direction: String,
    pub condition: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastDay {
    pub date: String,
    pub summary: String,
    pub low_c: f32,
    pub high_c: f32,
    pub rain_probability: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub title: String,
    pub severity: String,
    pub area: String,
    pub issued_at: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dashboard {
    pub place: Place,
    pub current: CurrentWeather,
    pub forecast: Vec<ForecastDay>,
    pub alerts: Vec<Alert>,
    pub source: DataSource,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    OfflineSample,
    PublicApi,
}

impl Dashboard {
    pub fn sample() -> Self {
        Self {
            place: Place {
                id: "manila".into(),
                name: "马尼拉大都会".into(),
                region: "菲律宾国家首都区".into(),
                latitude: 14.5995,
                longitude: 120.9842,
            },
            current: CurrentWeather {
                temperature_c: 30.0,
                feels_like_c: 34.0,
                humidity_percent: 72,
                wind_kph: 11.0,
                wind_direction: "西南".into(),
                condition: "局部多云".into(),
                updated_at: "离线示例".into(),
            },
            forecast: vec![
                ForecastDay {
                    date: "今天".into(),
                    summary: "局部阵雨".into(),
                    low_c: 26.0,
                    high_c: 32.0,
                    rain_probability: 55,
                },
                ForecastDay {
                    date: "明天".into(),
                    summary: "多云".into(),
                    low_c: 26.0,
                    high_c: 32.0,
                    rain_probability: 35,
                },
                ForecastDay {
                    date: "后天".into(),
                    summary: "短时阵雨".into(),
                    low_c: 25.0,
                    high_c: 31.0,
                    rain_probability: 60,
                },
                ForecastDay {
                    date: "周四".into(),
                    summary: "雷雨".into(),
                    low_c: 25.0,
                    high_c: 30.0,
                    rain_probability: 70,
                },
                ForecastDay {
                    date: "周五".into(),
                    summary: "局部多云".into(),
                    low_c: 26.0,
                    high_c: 32.0,
                    rain_probability: 40,
                },
            ],
            alerts: vec![Alert {
                title: "示例：强降雨提醒".into(),
                severity: "一般".into(),
                area: "马尼拉大都会".into(),
                issued_at: "离线示例".into(),
                detail: "联网后将从公开天气服务加载官方预警。".into(),
            }],
            source: DataSource::OfflineSample,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiVersion {
    pub latest_version: String,
    pub minimum_version: String,
}

#[cfg(feature = "network")]
pub fn fetch_api_version() -> Result<ApiVersion, String> {
    let response = ureq::get(&format!("{API_ROOT}/version"))
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "PanahonZh/0.1 (Rust clean-room client)")
        .call()
        .map_err(|err| format!("网络请求失败：{err}"))?;
    response
        .into_json::<ApiVersion>()
        .map_err(|err| format!("版本响应无法解析：{err}"))
}

#[cfg(not(feature = "network"))]
pub fn fetch_api_version() -> Result<ApiVersion, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
pub fn fetch_public_dashboard(place: &Place) -> Result<Dashboard, String> {
    let query = format!(
        "latitude={:.5}&longitude={:.5}",
        place.latitude, place.longitude
    );
    let nowcast = get_json(&format!("{API_ROOT}/nowcast/dashboard?{query}"))?;
    let observation = get_json(&format!("{API_ROOT}/observation/automatic?{query}"))?;
    let alerts = get_json(&format!("{API_ROOT}/alerts"))?;

    let now = nowcast
        .pointer("/data/nc")
        .or_else(|| nowcast.pointer("/data"));
    let obs = observation.pointer("/data/0/obs");
    let current = CurrentWeather {
        temperature_c: number(obs, "temp")
            .or_else(|| number(now, "tmp"))
            .unwrap_or(0.0) as f32,
        feels_like_c: number(obs, "heat_index")
            .or_else(|| number(now, "hindex"))
            .unwrap_or(0.0) as f32,
        humidity_percent: number(obs, "hum")
            .or_else(|| number(now, "hmax"))
            .unwrap_or(0.0)
            .clamp(0.0, 100.0) as u8,
        wind_kph: number(obs, "ws").unwrap_or(0.0) as f32,
        wind_direction: text(obs, "wd").unwrap_or_else(|| "未知".into()),
        condition: text(obs, "desc")
            .or_else(|| text(now, "desc"))
            .unwrap_or_else(|| "暂无描述".into()),
        updated_at: text(obs, "observed_at").unwrap_or_else(|| "公开接口".into()),
    };

    // `nowcast/dashboard` currently exposes hourly rows as `hc`; older/public
    // variants may expose daily rows as `dc`. Prefer daily rows when present,
    // then fall back to hourly rows so a transient API shape change does not
    // turn the forecast screen blank.
    let daily = nowcast
        .pointer("/data/dc")
        .and_then(serde_json::Value::as_array);
    let hourly = nowcast
        .pointer("/data/hc")
        .and_then(serde_json::Value::as_array);
    let forecast = daily
        .filter(|rows| !rows.is_empty())
        .map(|days| {
            days.iter()
                .take(5)
                .map(|day| ForecastDay {
                    date: text(Some(day), "d").unwrap_or_else(|| "未知日期".into()),
                    summary: text(Some(day), "desc").unwrap_or_else(|| "暂无描述".into()),
                    low_c: number(Some(day), "tmin").unwrap_or(0.0) as f32,
                    high_c: number(Some(day), "tmax").unwrap_or(0.0) as f32,
                    rain_probability: number(Some(day), "rain").unwrap_or(0.0).clamp(0.0, 100.0)
                        as u8,
                })
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            hourly.map(|rows| {
                rows.iter()
                    .take(5)
                    .map(|row| {
                        let temp = number(Some(row), "tmp").unwrap_or(0.0) as f32;
                        ForecastDay {
                            date: text(Some(row), "time").unwrap_or_else(|| "未知时间".into()),
                            summary: text(Some(row), "desc").unwrap_or_else(|| "暂无描述".into()),
                            low_c: temp,
                            high_c: temp,
                            rain_probability: number(Some(row), "rain")
                                .unwrap_or(0.0)
                                .clamp(0.0, 100.0)
                                as u8,
                        }
                    })
                    .collect::<Vec<_>>()
            })
        })
        .unwrap_or_default();

    let alert_items = alerts
        .pointer("/data")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(20)
                .map(|item| Alert {
                    title: text(Some(item), "title").unwrap_or_else(|| "天气预警".into()),
                    severity: text(Some(item), "type").unwrap_or_else(|| "一般".into()),
                    area: text(Some(item), "locations").unwrap_or_else(|| "未注明区域".into()),
                    issued_at: text(Some(item), "issued_at").unwrap_or_else(|| "未知时间".into()),
                    detail: text(Some(item), "message").unwrap_or_else(|| "请查看官方详情".into()),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Dashboard {
        place: place.clone(),
        current,
        forecast,
        alerts: alert_items,
        source: DataSource::PublicApi,
    })
}

#[cfg(not(feature = "network"))]
pub fn fetch_public_dashboard(_place: &Place) -> Result<Dashboard, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
pub fn search_places(query: &str) -> Result<Vec<Place>, String> {
    let query = query.trim();
    if query.chars().count() < 3 {
        return Err("请输入至少 3 个字符".into());
    }
    let encoded = percent_encode(query);
    let response = get_json(&format!("{API_ROOT}/location/search?q={encoded}"))?;
    let rows = response
        .as_array()
        .or_else(|| {
            response
                .pointer("/data")
                .and_then(serde_json::Value::as_array)
        })
        .ok_or_else(|| "地点响应格式无法解析".to_owned())?;
    Ok(rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            Some(Place {
                id: format!("location-{index}"),
                name: text(Some(row), "municipality")
                    .or_else(|| text(Some(row), "barangay"))
                    .unwrap_or_else(|| "未知地点".into()),
                region: text(Some(row), "region")
                    .or_else(|| text(Some(row), "province"))
                    .unwrap_or_else(|| "菲律宾".into()),
                latitude: number(Some(row), "lat")?,
                longitude: number(Some(row), "lon")?,
            })
        })
        .collect())
}

#[cfg(not(feature = "network"))]
pub fn search_places(_query: &str) -> Result<Vec<Place>, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
fn get_json(url: &str) -> Result<serde_json::Value, String> {
    ureq::get(url)
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "PanahonZh/0.1 (Rust clean-room client)")
        .call()
        .map_err(|err| format!("网络请求失败：{err}"))?
        .into_json::<serde_json::Value>()
        .map_err(|err| format!("接口响应无法解析：{err}"))
}

#[cfg(feature = "network")]
fn number(value: Option<&serde_json::Value>, key: &str) -> Option<f64> {
    let item = value?.get(key)?;
    item.as_f64()
        .or_else(|| item.as_str().and_then(parse_number))
}

#[cfg(feature = "network")]
fn text(value: Option<&serde_json::Value>, key: &str) -> Option<String> {
    let item = value?.get(key)?;
    item.as_str()
        .map(str::to_owned)
        .or_else(|| {
            item.as_array().map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect::<Vec<_>>()
                    .join("、")
            })
        })
        .or_else(|| Some(item.to_string()))
}

#[cfg(feature = "network")]
fn parse_number(value: &str) -> Option<f64> {
    let start = value
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_digit() || *ch == '-' || *ch == '.')
        .map(|(index, _)| index)?;
    let tail = &value[start..];
    let end = tail
        .char_indices()
        .skip_while(|(_, ch)| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .map(|(index, _)| index)
        .next()
        .unwrap_or(tail.len());
    tail[..end].parse().ok()
}

#[cfg(feature = "network")]
fn percent_encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                char::from(*byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_dashboard_has_five_days_and_a_place() {
        let dashboard = Dashboard::sample();
        assert_eq!(dashboard.forecast.len(), 5);
        assert_eq!(dashboard.place.id, "manila");
        assert_eq!(dashboard.source, DataSource::OfflineSample);
    }

    #[test]
    fn version_json_is_stable() {
        let parsed: ApiVersion =
            serde_json::from_str(r#"{"latest_version":"1.0.4","minimum_version":"1.0.4"}"#)
                .expect("fixture is valid JSON");
        assert_eq!(parsed.latest_version, "1.0.4");
    }

    #[cfg(feature = "network")]
    #[test]
    fn public_units_are_parsed_without_becoming_zero() {
        assert_eq!(parse_number("28 °C"), Some(28.0));
        assert_eq!(parse_number("72.8%"), Some(72.8));
        assert_eq!(parse_number("0.3 km/h"), Some(0.3));
        assert_eq!(parse_number("-1 °C"), Some(-1.0));
    }
}
