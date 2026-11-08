//! Stable data boundary for the clean-room PANaHON rebuild.
//! The app is usable offline and treats the public API as an optional adapter.

use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::io::Read;
#[cfg(feature = "network")]
use std::time::Duration;

pub const API_ROOT: &str = "https://api.mobile.panahon.gov.ph/api/v1";
pub const MAP_ROOT: &str = "https://origin.panahon.gov.ph";
pub const OFFICIAL_MAP_URL: &str = "https://panahon.gov.ph/?embed=panahon-mobile";
pub const AQI_ENDPOINT: &str = match option_env!("PANAHON_AQI_ENDPOINT") {
    Some(value) => value,
    None => "https://aqi.example.invalid/api/aqi",
};

/// A read-only snapshot of one public PANaHON map layer.
///
/// The image is the latest frame for radar or satellite. Point and warning
/// layers expose a count instead because their official endpoint returns
/// vector data rather than a single raster image.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteMapSnapshot {
    pub layer: String,
    pub latest_at: Option<String>,
    pub frame_count: Option<u32>,
    pub item_count: Option<u32>,
    pub image: Option<Vec<u8>>,
}

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
pub struct HourlyForecast {
    pub time: String,
    pub summary: String,
    pub temperature_c: f32,
    pub rain_probability: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub title: String,
    pub severity: String,
    pub area: String,
    pub issued_at: String,
    pub detail: String,
    pub defense: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirQualityStation {
    #[serde(rename = "nameZh")]
    pub name_zh: String,
    pub pm10: Option<f32>,
    #[serde(rename = "pm25")]
    pub pm25: Option<f32>,
    pub aqi: Option<u16>,
    pub pollutant: Option<String>,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirQualitySummary {
    #[serde(rename = "stationCount")]
    pub station_count: u16,
    #[serde(rename = "avgAqi")]
    pub avg_aqi: Option<u16>,
    #[serde(rename = "maxAqi")]
    pub max_aqi: Option<u16>,
    #[serde(rename = "maxStation")]
    pub max_station: Option<String>,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirQuality {
    pub source: AirQualitySource,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "dataClock")]
    pub data_clock: Option<String>,
    #[serde(rename = "fetchedAt")]
    pub fetched_at: Option<String>,
    pub summary: AirQualitySummary,
    pub stations: Vec<AirQualityStation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirQualitySource {
    pub provider: String,
    pub report: Option<String>,
    pub report_id: Option<u64>,
    pub page: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dashboard {
    pub place: Place,
    pub current: CurrentWeather,
    pub forecast: Vec<ForecastDay>,
    pub hourly: Vec<HourlyForecast>,
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
            hourly: vec![
                HourlyForecast {
                    time: "现在".into(),
                    summary: "局部多云".into(),
                    temperature_c: 30.0,
                    rain_probability: 20,
                },
                HourlyForecast {
                    time: "1 小时后".into(),
                    summary: "局部多云".into(),
                    temperature_c: 30.0,
                    rain_probability: 25,
                },
                HourlyForecast {
                    time: "2 小时后".into(),
                    summary: "短时阵雨".into(),
                    temperature_c: 29.0,
                    rain_probability: 45,
                },
                HourlyForecast {
                    time: "3 小时后".into(),
                    summary: "短时阵雨".into(),
                    temperature_c: 29.0,
                    rain_probability: 50,
                },
                HourlyForecast {
                    time: "4 小时后".into(),
                    summary: "多云".into(),
                    temperature_c: 29.0,
                    rain_probability: 35,
                },
                HourlyForecast {
                    time: "5 小时后".into(),
                    summary: "多云".into(),
                    temperature_c: 28.0,
                    rain_probability: 30,
                },
            ],
            alerts: vec![Alert {
                title: "示例：强降雨提醒".into(),
                severity: "一般".into(),
                area: "马尼拉大都会".into(),
                issued_at: "离线示例".into(),
                detail: "联网后将从公开天气服务加载官方预警。".into(),
                defense: "联网并确认数据来源后，再按官方防御指南行动。".into(),
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
pub fn fetch_air_quality() -> Result<AirQuality, String> {
    ureq::get(AQI_ENDPOINT)
        .timeout(Duration::from_secs(15))
        .set("User-Agent", "PanahonZh/0.1 (Rust client)")
        .call()
        .map_err(|err| format!("空气质量服务请求失败：{err}"))?
        .into_json::<AirQuality>()
        .map_err(|err| format!("空气质量响应无法解析：{err}"))
}

#[cfg(not(feature = "network"))]
pub fn fetch_air_quality() -> Result<AirQuality, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
pub fn fetch_remote_map_layer(layer: &str) -> Result<RemoteMapSnapshot, String> {
    let token = fetch_map_token()?;
    match layer {
        "radar" => fetch_radar_snapshot(&token),
        "satellite" => fetch_satellite_snapshot(&token),
        "lightning" => {
            let value = map_json(&format!(
                "{MAP_ROOT}/api/v1/lightning?token={token}&parameter=ten_minute_frequency"
            ))?;
            let rows = value
                .pointer("/data")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "闪电图层响应格式无法解析".to_owned())?;
            Ok(RemoteMapSnapshot {
                layer: layer.to_owned(),
                latest_at: rows
                    .iter()
                    .rev()
                    .find_map(|row| row.get("observed_at").and_then(serde_json::Value::as_str))
                    .map(str::to_owned),
                frame_count: None,
                item_count: Some(rows.len() as u32),
                image: None,
            })
        }
        "typhoon" => {
            let value = map_json(&format!("{MAP_ROOT}/api/v1/cyclone-track?token={token}"))?;
            let rows = value
                .as_array()
                .ok_or_else(|| "台风图层响应格式无法解析".to_owned())?;
            let latest_at = rows
                .iter()
                .filter_map(|row| row.get("info").and_then(serde_json::Value::as_object))
                .flat_map(|info| info.keys())
                .max()
                .cloned();
            Ok(RemoteMapSnapshot {
                layer: layer.to_owned(),
                latest_at,
                frame_count: None,
                item_count: Some(rows.len() as u32),
                image: None,
            })
        }
        "hazard" => {
            let value = map_json(&format!("{MAP_ROOT}/api/v1/cap-alerts?token={token}"))?;
            let rows = value
                .pointer("/data/alert_data")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "灾害预警图层响应格式无法解析".to_owned())?;
            Ok(RemoteMapSnapshot {
                layer: layer.to_owned(),
                latest_at: rows
                    .iter()
                    .filter_map(|row| row.get("issued_date").and_then(serde_json::Value::as_str))
                    .max()
                    .map(str::to_owned),
                frame_count: None,
                item_count: Some(
                    value
                        .pointer("/data/alert_count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(rows.len() as u64) as u32,
                ),
                image: None,
            })
        }
        _ => Err(format!("未知地图图层：{layer}")),
    }
}

#[cfg(not(feature = "network"))]
pub fn fetch_remote_map_layer(_layer: &str) -> Result<RemoteMapSnapshot, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
fn fetch_map_token() -> Result<String, String> {
    let html = ureq::get(MAP_ROOT)
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "PanahonZh/0.1 (Rust client)")
        .call()
        .map_err(|err| format!("PAGASA 地图首页请求失败：{err}"))?
        .into_string()
        .map_err(|err| format!("PAGASA 地图首页无法读取：{err}"))?;
    let marker = r#"name="csrf-token" content=""#;
    let start = html
        .find(marker)
        .map(|index| index + marker.len())
        .ok_or_else(|| "PAGASA 地图首页没有返回 CSRF 令牌".to_owned())?;
    let token = html[start..].split('"').next().unwrap_or_default().trim();
    if token.is_empty() {
        return Err("PAGASA 地图首页返回了空 CSRF 令牌".into());
    }
    Ok(token.to_owned())
}

#[cfg(feature = "network")]
fn fetch_radar_snapshot(token: &str) -> Result<RemoteMapSnapshot, String> {
    let sublayer = "mosaic-rainrate";
    let value = map_json(&format!(
        "{MAP_ROOT}/api/v1/radar/timeline?token={token}&sublayer={sublayer}"
    ))?;
    let rows = value
        .pointer("/data/timeline")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "雷达图层响应格式无法解析".to_owned())?;
    let latest = rows
        .last()
        .ok_or_else(|| "雷达图层没有可用时间帧".to_owned())?;
    let timestamp = latest
        .get("observed_at_unix")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| "雷达图层缺少时间戳".to_owned())?;
    let observed_at = latest
        .get("observed_at")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let version = value
        .pointer("/data/tile_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    let mode = value
        .pointer("/data/scale/mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("rain");
    let image_url = format!(
        "{MAP_ROOT}/api/v1/radar-data-image?token={token}&sublayer={sublayer}&t={timestamp}&mode={mode}&size=1024&v={version}"
    );
    Ok(RemoteMapSnapshot {
        layer: "radar".into(),
        latest_at: observed_at,
        frame_count: Some(rows.len() as u32),
        item_count: None,
        image: Some(map_bytes(&image_url)?),
    })
}

#[cfg(feature = "network")]
fn fetch_satellite_snapshot(token: &str) -> Result<RemoteMapSnapshot, String> {
    let value = map_json(&format!(
        "{MAP_ROOT}/api/v1/himawari/timeline?token={token}"
    ))?;
    let rows = value
        .pointer("/data/timeline")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "卫星图层响应格式无法解析".to_owned())?;
    let latest = rows
        .last()
        .ok_or_else(|| "卫星图层没有可用时间帧".to_owned())?;
    let index = rows.len().saturating_sub(1);
    let revision = latest
        .get("revision")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0");
    let version = value
        .pointer("/data/tile_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    let image_url = format!(
        "{MAP_ROOT}/api/v1/himawari-data?index={index}&size=1024&v={version}-{revision}&token={token}&parameter=himawari_ir_ext_smooth"
    );
    Ok(RemoteMapSnapshot {
        layer: "satellite".into(),
        latest_at: latest
            .get("observed_at")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        frame_count: Some(rows.len() as u32),
        item_count: None,
        image: Some(map_bytes(&image_url)?),
    })
}

#[cfg(feature = "network")]
fn map_json(url: &str) -> Result<serde_json::Value, String> {
    ureq::get(url)
        .timeout(Duration::from_secs(20))
        .set("User-Agent", "PanahonZh/0.1 (Rust client)")
        .set("Accept", "application/json")
        .call()
        .map_err(|err| format!("PAGASA 地图接口请求失败：{err}"))?
        .into_json::<serde_json::Value>()
        .map_err(|err| format!("PAGASA 地图响应无法解析：{err}"))
}

#[cfg(feature = "network")]
fn map_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    ureq::get(url)
        .timeout(Duration::from_secs(30))
        .set("User-Agent", "PanahonZh/0.1 (Rust client)")
        .set("Accept", "image/png")
        .call()
        .map_err(|err| format!("PAGASA 地图图像请求失败：{err}"))?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("PAGASA 地图图像读取失败：{err}"))?;
    if bytes.is_empty() {
        return Err("PAGASA 地图图像为空".into());
    }
    Ok(bytes)
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
            .map(translate_condition)
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
    let hourly_items = hourly
        .map(|rows| {
            rows.iter()
                .take(24)
                .map(|row| HourlyForecast {
                    time: text(Some(row), "time")
                        .map(translate_day)
                        .unwrap_or_else(|| "未知时间".into()),
                    summary: text(Some(row), "desc")
                        .map(translate_condition)
                        .unwrap_or_else(|| "暂无描述".into()),
                    temperature_c: number(Some(row), "tmp").unwrap_or(0.0) as f32,
                    rain_probability: number(Some(row), "rain").unwrap_or(0.0).clamp(0.0, 100.0)
                        as u8,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let forecast = daily
        .filter(|rows| !rows.is_empty())
        .map(|days| {
            days.iter()
                .take(5)
                .map(|day| ForecastDay {
                    date: text(Some(day), "d")
                        .map(translate_day)
                        .unwrap_or_else(|| "未知日期".into()),
                    summary: text(Some(day), "desc")
                        .map(translate_condition)
                        .unwrap_or_else(|| "暂无描述".into()),
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
                            date: text(Some(row), "time")
                                .map(translate_day)
                                .unwrap_or_else(|| "未知时间".into()),
                            summary: text(Some(row), "desc")
                                .map(translate_condition)
                                .unwrap_or_else(|| "暂无描述".into()),
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
                    defense: "请以官方预警和当地部门指引为准。".into(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Dashboard {
        place: place.clone(),
        current,
        forecast,
        hourly: hourly_items,
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
    if item.is_null() {
        return None;
    }
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

#[cfg(feature = "network")]
fn translate_condition(value: String) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("clear") {
        "晴朗".into()
    } else if lower.contains("partly cloudy") {
        "局部多云".into()
    } else if lower.contains("cloudy") || lower.contains("overcast") {
        "多云".into()
    } else if lower.contains("thunder") || lower.contains("storm") {
        "雷雨".into()
    } else if lower.contains("shower") || lower.contains("rain") {
        "阵雨".into()
    } else if lower.contains("fair") {
        "晴间多云".into()
    } else {
        value
    }
}

#[cfg(feature = "network")]
fn translate_day(value: String) -> String {
    match value.to_ascii_lowercase().as_str() {
        "today" => "今天".into(),
        "sun" | "sunday" => "周日".into(),
        "mon" | "monday" => "周一".into(),
        "tue" | "tuesday" => "周二".into(),
        "wed" | "wednesday" => "周三".into(),
        "thu" | "thursday" => "周四".into(),
        "fri" | "friday" => "周五".into(),
        "sat" | "saturday" => "周六".into(),
        _ => value,
    }
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
