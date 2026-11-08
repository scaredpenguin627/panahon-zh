//! Stable data boundary for the clean-room PANaHON rebuild.
//!
//! The app is usable offline: [`Dashboard::sample`] is a complete snapshot that
//! every screen can render. Network adapters are optional and only extend the
//! model, never replace it.

use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::io::Read;
#[cfg(feature = "network")]
use std::time::Duration;

#[cfg(feature = "network")]
pub mod emb;

pub const API_ROOT: &str = "https://api.mobile.panahon.gov.ph/api/v1";
pub const MAP_ROOT: &str = "https://origin.panahon.gov.ph";
pub const OFFICIAL_MAP_URL: &str = "https://panahon.gov.ph/?embed=panahon-mobile";
pub const AQI_ENDPOINT: &str = match option_env!("PANAHON_AQI_ENDPOINT") {
    Some(value) => value,
    None => "",
};

/// A coarse weather glyph. The UI draws these as vectors, so the same enum
/// covers PAGASA's icon filenames, offline samples and future sources.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WeatherIcon {
    Clear,
    PartlyCloudy,
    Cloudy,
    RainShowers,
    Rain,
    HeavyRain,
    Thunderstorm,
    Fog,
    Wind,
    Unknown,
}

impl WeatherIcon {
    /// Map an English PAGASA description or icon filename to a glyph.
    pub fn from_text(text: &str) -> Self {
        let text = text.to_ascii_lowercase();
        if text.contains("thunder") {
            Self::Thunderstorm
        } else if text.contains("heavy rain") || text.contains("torrential") {
            Self::HeavyRain
        } else if text.contains("rainshower") || text.contains("shower") {
            Self::RainShowers
        } else if text.contains("rain") {
            Self::Rain
        } else if text.contains("fog") || text.contains("mist") || text.contains("haze") {
            Self::Fog
        } else if text.contains("partly cloudy") || text.contains("cloudy to at times") {
            Self::PartlyCloudy
        } else if text.contains("cloudy") || text.contains("overcast") {
            Self::Cloudy
        } else if text.contains("clear") || text.contains("sunny") || text.contains("fair") {
            Self::Clear
        } else if text.contains("wind") {
            Self::Wind
        } else {
            Self::Unknown
        }
    }

    /// Short Chinese label, used by the accessibility text and the sample data.
    pub fn label(self) -> &'static str {
        match self {
            Self::Clear => "晴",
            Self::PartlyCloudy => "多云",
            Self::Cloudy => "阴",
            Self::RainShowers => "阵雨",
            Self::Rain => "小雨",
            Self::HeavyRain => "大雨",
            Self::Thunderstorm => "雷雨",
            Self::Fog => "雾",
            Self::Wind => "大风",
            Self::Unknown => "未知",
        }
    }

    /// True when the glyph should be drawn with a rain or storm accent.
    pub fn is_wet(self) -> bool {
        matches!(
            self,
            Self::RainShowers | Self::Rain | Self::HeavyRain | Self::Thunderstorm
        )
    }
}

/// A read-only snapshot of one public PANaHON map layer.
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
    pub icon: WeatherIcon,
    pub is_day: bool,
    /// Total precipitation in the latest hour, in millimetres.
    pub precip_mm: f32,
    pub precip_desc: String,
    /// PAGASA heat-index band, e.g. "Caution". Empty when unknown.
    pub heat_index_desc: String,
    pub pressure_hpa: Option<f32>,
    pub solar_wm2: Option<f32>,
    /// Automatic station the observation came from.
    pub station: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastDay {
    /// Already-localised label: 今天、明天、周四 …
    pub date: String,
    /// Raw PAGASA date, e.g. 11/08.
    pub date_label: String,
    pub summary: String,
    pub low_c: f32,
    pub high_c: f32,
    pub precip_mm: f32,
    pub icon: WeatherIcon,
    pub heat_index_c: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HourlyForecast {
    /// Already-localised hour label: 现在、13时 …
    pub time: String,
    pub summary: String,
    pub temperature_c: f32,
    pub precip_mm: f32,
    pub icon: WeatherIcon,
    pub is_day: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub id: String,
    pub title: String,
    /// PAGASA alert type, e.g. "Rainfall Advisory".
    pub kind: String,
    pub area: String,
    pub issued_at: String,
    pub valid_until: String,
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

/// A locally derived suggestion. No extra API call is involved.
#[derive(Clone, Debug, PartialEq)]
pub struct LifeIndex {
    pub title: String,
    pub level: String,
    pub advice: String,
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

/// AQI 0–500 split into the six bands used by PAGASA and DENR-EMB.
pub fn aqi_category(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "优",
        51..=100 => "良",
        101..=150 => "轻度污染",
        151..=200 => "中度污染",
        201..=300 => "重度污染",
        _ => "严重污染",
    }
}

impl Dashboard {
    /// Offline snapshot: every screen stays usable with networking disabled.
    pub fn sample() -> Self {
        Self {
            place: Place {
                id: "manila".into(),
                name: "马尼拉".into(),
                region: "国家首都区".into(),
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
                icon: WeatherIcon::PartlyCloudy,
                is_day: true,
                precip_mm: 0.0,
                precip_desc: "无降雨".into(),
                heat_index_desc: "注意".into(),
                pressure_hpa: Some(1010.0),
                solar_wm2: Some(520.0),
                station: Some("示例站点".into()),
            },
            forecast: vec![
                day(
                    "今天",
                    "11/08",
                    "局部阵雨",
                    26.0,
                    32.0,
                    1.4,
                    WeatherIcon::RainShowers,
                    Some(36.0),
                ),
                day(
                    "明天",
                    "11/09",
                    "多云",
                    26.0,
                    32.0,
                    0.2,
                    WeatherIcon::PartlyCloudy,
                    Some(35.0),
                ),
                day(
                    "后天",
                    "11/10",
                    "阵雨",
                    25.0,
                    31.0,
                    2.1,
                    WeatherIcon::RainShowers,
                    Some(34.0),
                ),
                day(
                    "周三",
                    "11/11",
                    "雷雨",
                    25.0,
                    30.0,
                    6.8,
                    WeatherIcon::Thunderstorm,
                    Some(33.0),
                ),
                day(
                    "周四",
                    "11/12",
                    "局部多云",
                    26.0,
                    32.0,
                    0.4,
                    WeatherIcon::PartlyCloudy,
                    Some(35.0),
                ),
            ],
            hourly: (0..24)
                .map(|index| {
                    let hour = (12 + index) % 24;
                    let afternoon = (12..18).contains(&hour);
                    HourlyForecast {
                        time: if index == 0 {
                            "现在".into()
                        } else {
                            format!("{hour}时")
                        },
                        summary: if afternoon { "局部多云" } else { "多云" }.into(),
                        temperature_c: if afternoon { 32.0 } else { 27.0 },
                        precip_mm: if afternoon { 0.2 } else { 0.0 },
                        icon: if afternoon {
                            WeatherIcon::PartlyCloudy
                        } else {
                            WeatherIcon::Cloudy
                        },
                        is_day: (6..18).contains(&hour),
                    }
                })
                .collect(),
            alerts: vec![Alert {
                id: "sample-1".into(),
                title: "示例：强降雨提醒".into(),
                kind: "Rainfall Advisory".into(),
                area: "马尼拉大都会".into(),
                issued_at: "离线示例".into(),
                valid_until: "离线示例".into(),
                detail: "联网后将从 PAGASA 公开接口加载官方预警。".into(),
                defense: "联网并确认数据来源后，再按官方防御指南行动。".into(),
            }],
            source: DataSource::OfflineSample,
        }
    }

    /// Derive four suggestions from the current observation.
    pub fn life_indices(&self) -> Vec<LifeIndex> {
        life_indices(&self.current)
    }
}

fn day(
    date: &str,
    date_label: &str,
    summary: &str,
    low_c: f32,
    high_c: f32,
    precip_mm: f32,
    icon: WeatherIcon,
    heat_index_c: Option<f32>,
) -> ForecastDay {
    ForecastDay {
        date: date.into(),
        date_label: date_label.into(),
        summary: summary.into(),
        low_c,
        high_c,
        precip_mm,
        icon,
        heat_index_c,
    }
}

pub fn life_indices(current: &CurrentWeather) -> Vec<LifeIndex> {
    let temp = current.temperature_c;
    let (wear_level, wear_advice) = if temp >= 32.0 {
        ("炎热", "轻薄短袖，注意防晒补水")
    } else if temp >= 27.0 {
        ("较热", "短袖即可，室内注意通风")
    } else if temp >= 22.0 {
        ("舒适", "短袖或薄长袖都合适")
    } else if temp >= 16.0 {
        ("偏凉", "建议加一件薄外套")
    } else {
        ("寒冷", "穿厚外套，注意保暖")
    };

    let feels = current.feels_like_c.max(temp);
    let (heat_level, heat_advice) = if feels >= 54.0 {
        ("危险", "尽量避免户外活动，及时补水")
    } else if feels >= 41.0 {
        ("谨慎", "减少长时间暴晒，随身带水")
    } else if feels >= 32.0 {
        ("注意", "长时间户外活动要多补水")
    } else {
        ("舒适", "体感舒适，适合户外活动")
    };

    let (trip_level, trip_advice) = if current.precip_mm >= 5.0 {
        ("不宜", "有明显降雨，带伞并留意积水")
    } else if current.precip_mm > 0.2 {
        ("一般", "可能有阵雨，出门带伞")
    } else {
        ("适宜", "降雨概率低，适合出行")
    };

    let (dry_level, dry_advice) = if current.humidity_percent >= 85 || current.precip_mm > 0.2 {
        ("不宜", "湿度大或有降雨，不建议晾晒")
    } else if current.humidity_percent >= 70 {
        ("一般", "晾晒时间需要拉长")
    } else {
        ("适宜", "空气较干，适合晾晒")
    };

    vec![
        LifeIndex {
            title: "穿衣".into(),
            level: wear_level.into(),
            advice: wear_advice.into(),
        },
        LifeIndex {
            title: "中暑".into(),
            level: heat_level.into(),
            advice: heat_advice.into(),
        },
        LifeIndex {
            title: "出行".into(),
            level: trip_level.into(),
            advice: trip_advice.into(),
        },
        LifeIndex {
            title: "晾晒".into(),
            level: dry_level.into(),
            advice: dry_advice.into(),
        },
    ]
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
        .set("User-Agent", "PanahonZh/0.2 (Rust clean-room client)")
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

/// Air quality, in order of preference:
/// 1. an explicit `PANAHON_AQI_ENDPOINT` proxy, when the build sets one;
/// 2. DENR-EMB's official Power BI report, read directly;
/// 3. Open-Meteo CAMS as a position-level fallback.
#[cfg(feature = "network")]
pub fn fetch_air_quality() -> Result<AirQuality, String> {
    fetch_air_quality_at(14.5995, 120.9842)
}

#[cfg(feature = "network")]
pub fn fetch_air_quality_at(latitude: f64, longitude: f64) -> Result<AirQuality, String> {
    if !AQI_ENDPOINT.is_empty() {
        return ureq::get(AQI_ENDPOINT)
            .timeout(Duration::from_secs(20))
            .set("User-Agent", "PanahonZh/0.2 (Rust client)")
            .call()
            .map_err(|err| format!("空气质量服务请求失败：{err}"))?
            .into_json::<AirQuality>()
            .map_err(|err| format!("空气质量响应无法解析：{err}"));
    }

    match emb::fetch() {
        Ok(data) => Ok(data),
        Err(emb_error) => {
            let mut fallback = fetch_cams_air_quality(latitude, longitude)?;
            fallback.source.note =
                format!("DENR-EMB 官方报表暂时读不到，已改用位置级预测。原因：{emb_error}");
            Ok(fallback)
        }
    }
}

#[cfg(feature = "network")]
fn fetch_cams_air_quality(latitude: f64, longitude: f64) -> Result<AirQuality, String> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={latitude:.4}&longitude={longitude:.4}&current=pm10,pm2_5,us_aqi&timezone=auto"
    );
    let value = get_json(&url)?;
    let current = value
        .pointer("/current")
        .ok_or_else(|| "Open-Meteo 空气质量响应缺少 current".to_owned())?;
    let aqi = number(Some(current), "us_aqi").map(|value| value.round().clamp(0.0, 500.0) as u16);
    let category = aqi
        .map(|value| aqi_category(value).to_string())
        .unwrap_or_else(|| "暂无".into());
    let data_clock = text(Some(current), "time");
    let station = AirQualityStation {
        name_zh: "当前位置（预测）".into(),
        pm10: number(Some(current), "pm10").map(|value| value as f32),
        pm25: number(Some(current), "pm2_5").map(|value| value as f32),
        aqi,
        pollutant: None,
        category: category.clone(),
    };
    Ok(AirQuality {
        source: AirQualitySource {
            provider: "Open-Meteo / CAMS".into(),
            report: Some("CAMS 全球空气质量预测".into()),
            report_id: None,
            page: "https://open-meteo.com/en/docs/air-quality-api".into(),
            note: "位置级预测。DENR-EMB 官方站点数据不可用时的备用来源。".into(),
        },
        updated_at: data_clock.clone(),
        data_clock,
        fetched_at: None,
        summary: AirQualitySummary {
            station_count: 1,
            avg_aqi: aqi,
            max_aqi: aqi,
            max_station: Some(station.name_zh.clone()),
            category: Some(category),
        },
        stations: vec![station],
    })
}

#[cfg(not(feature = "network"))]
pub fn fetch_air_quality() -> Result<AirQuality, String> {
    Err("网络适配器未启用".into())
}

#[cfg(not(feature = "network"))]
pub fn fetch_air_quality_at(_latitude: f64, _longitude: f64) -> Result<AirQuality, String> {
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
        .set("User-Agent", "PanahonZh/0.2 (Rust client)")
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
        .set("User-Agent", "PanahonZh/0.2 (Rust client)")
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
        .set("User-Agent", "PanahonZh/0.2 (Rust client)")
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

    let icon_source = text(obs, "icon").or_else(|| text(now, "icon"));
    let description = text(obs, "desc").or_else(|| text(now, "desc"));
    let icon = description
        .as_deref()
        .map(WeatherIcon::from_text)
        .or_else(|| icon_source.as_deref().map(WeatherIcon::from_text))
        .unwrap_or(WeatherIcon::Unknown);

    let temperature = number(obs, "temp")
        .or_else(|| number(now, "tmp"))
        .unwrap_or(0.0) as f32;
    let feels_like = number(obs, "heat_index")
        .or_else(|| number(now, "hindex"))
        .unwrap_or(temperature as f64) as f32;

    let current = CurrentWeather {
        temperature_c: temperature,
        feels_like_c: feels_like,
        humidity_percent: number(obs, "hum").unwrap_or(0.0).clamp(0.0, 100.0) as u8,
        wind_kph: number(obs, "ws").unwrap_or(0.0) as f32,
        wind_direction: text(obs, "wd")
            .map(|value| wind_direction_zh(&value))
            .unwrap_or_else(|| "未知".into()),
        condition: description
            .clone()
            .map(translate_condition)
            .unwrap_or_else(|| "暂无描述".into()),
        updated_at: text(obs, "observed_at").unwrap_or_else(|| "公开接口".into()),
        icon,
        is_day: icon_source.as_deref().map(is_day_icon).unwrap_or(true),
        precip_mm: number(obs, "precip_1h")
            .or_else(|| number(now, "tprecip"))
            .unwrap_or(0.0) as f32,
        precip_desc: text(now, "tprecip_desc")
            .map(|value| translate_precip(&value))
            .unwrap_or_else(|| "无降雨".into()),
        heat_index_desc: text(now, "hindex_desc")
            .map(|value| translate_heat_index(&value))
            .unwrap_or_default(),
        pressure_hpa: number(obs, "mslp").map(|value| value as f32),
        solar_wm2: number(obs, "sr").map(|value| value as f32),
        station: text(obs, "loc"),
    };

    let daily = nowcast
        .pointer("/data/dc")
        .and_then(serde_json::Value::as_array);
    let hourly = nowcast
        .pointer("/data/hc")
        .and_then(serde_json::Value::as_array);

    let forecast = daily
        .map(|days| {
            days.iter()
                .take(5)
                .map(|row| {
                    let description = text(Some(row), "desc").unwrap_or_default();
                    let icon = text(Some(row), "icon")
                        .as_deref()
                        .map(WeatherIcon::from_text)
                        .unwrap_or_else(|| WeatherIcon::from_text(&description));
                    ForecastDay {
                        date: text(Some(row), "d")
                            .map(translate_day)
                            .unwrap_or_else(|| "未知".into()),
                        date_label: text(Some(row), "dt").unwrap_or_default(),
                        summary: translate_condition(description),
                        low_c: number(Some(row), "tmin").unwrap_or(0.0) as f32,
                        high_c: number(Some(row), "tmax").unwrap_or(0.0) as f32,
                        precip_mm: number(Some(row), "tprecip").unwrap_or(0.0) as f32,
                        icon,
                        heat_index_c: number(Some(row), "hmax").map(|value| value as f32),
                    }
                })
                .collect::<Vec<_>>()
        })
        .filter(|rows| !rows.is_empty())
        .unwrap_or_default();

    let hourly_items = hourly
        .map(|rows| {
            rows.iter()
                .take(24)
                .enumerate()
                .map(|(index, row)| {
                    let description = text(Some(row), "desc").unwrap_or_default();
                    let icon_url = text(Some(row), "icon");
                    let icon = icon_url
                        .as_deref()
                        .map(WeatherIcon::from_text)
                        .unwrap_or_else(|| WeatherIcon::from_text(&description));
                    HourlyForecast {
                        time: text(Some(row), "time")
                            .map(|value| hour_label(&value, index == 0))
                            .unwrap_or_else(|| "未知".into()),
                        summary: translate_condition(description),
                        temperature_c: number(Some(row), "tmp").unwrap_or(0.0) as f32,
                        precip_mm: number(Some(row), "tprecip").unwrap_or(0.0) as f32,
                        icon,
                        is_day: icon_url.as_deref().map(is_day_icon).unwrap_or(true),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let alert_items = alerts
        .pointer("/data")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(20)
                .map(|item| {
                    let kind = text(Some(item), "type").unwrap_or_else(|| "天气预警".into());
                    let raw_title = text(Some(item), "title").unwrap_or_else(|| "天气预警".into());
                    Alert {
                        id: text(Some(item), "alert_id").unwrap_or_default(),
                        title: translate_alert_title(&raw_title),
                        kind: translate_alert_kind(&kind),
                        area: text(Some(item), "locations").unwrap_or_else(|| "未注明区域".into()),
                        issued_at: text(Some(item), "issued_at")
                            .unwrap_or_else(|| "未知时间".into()),
                        valid_until: text(Some(item), "valid_until_at").unwrap_or_default(),
                        detail: text(Some(item), "message")
                            .unwrap_or_else(|| "请查看官方详情".into()),
                        defense: defense_for(&kind).into(),
                    }
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
            let name = text(Some(row), "barangay")
                .or_else(|| text(Some(row), "municipality"))
                .or_else(|| text(Some(row), "province"))?;
            Some(Place {
                id: format!("location-{index}"),
                name,
                region: text(Some(row), "province")
                    .or_else(|| text(Some(row), "region"))
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
        .set("User-Agent", "PanahonZh/0.2 (Rust clean-room client)")
        .call()
        .map_err(|err| format!("网络请求失败：{err}"))?
        .into_json::<serde_json::Value>()
        .map_err(|err| format!("接口响应无法解析：{err}"))
}

#[cfg(feature = "network")]
pub(crate) fn number(value: Option<&serde_json::Value>, key: &str) -> Option<f64> {
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
fn is_day_icon(url: &str) -> bool {
    !url.contains("/pm/")
}

#[cfg(feature = "network")]
pub(crate) fn translate_condition(value: String) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("thunder") {
        "雷雨".into()
    } else if lower.contains("rainshower") || lower.contains("shower") {
        "阵雨".into()
    } else if lower.contains("heavy rain") {
        "大雨".into()
    } else if lower.contains("rain") {
        "小雨".into()
    } else if lower.contains("partly cloudy") {
        "局部多云".into()
    } else if lower.contains("cloudy") || lower.contains("overcast") {
        "多云".into()
    } else if lower.contains("clear") {
        "晴".into()
    } else if lower.contains("fair") {
        "晴间多云".into()
    } else if lower.contains("fog") || lower.contains("mist") {
        "雾".into()
    } else {
        value
    }
}

#[cfg(feature = "network")]
fn translate_precip(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("no rain") {
        "无降雨".into()
    } else if lower.contains("very light") {
        "微量降雨".into()
    } else if lower.contains("light") {
        "小雨".into()
    } else if lower.contains("moderate") {
        "中雨".into()
    } else if lower.contains("heavy") {
        "大雨".into()
    } else if lower.contains("intense") || lower.contains("torrential") {
        "暴雨".into()
    } else {
        value.to_owned()
    }
}

#[cfg(feature = "network")]
fn translate_heat_index(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("extreme") {
        "极易中暑".into()
    } else if lower.contains("danger") {
        "有中暑危险".into()
    } else if lower.contains("extreme caution") {
        "注意防暑".into()
    } else if lower.contains("caution") {
        "注意补水".into()
    } else {
        value.to_owned()
    }
}

#[cfg(feature = "network")]
fn translate_alert_kind(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("rainfall") {
        "降雨提示".into()
    } else if lower.contains("thunderstorm") {
        "雷暴提示".into()
    } else if lower.contains("flood") {
        "洪水提示".into()
    } else if lower.contains("tropical cyclone") || lower.contains("typhoon") {
        "台风提示".into()
    } else if lower.contains("gale") || lower.contains("wind") {
        "大风提示".into()
    } else if lower.contains("heat") {
        "高温提示".into()
    } else {
        value.to_owned()
    }
}

#[cfg(feature = "network")]
/// PAGASA 的标题是英文编号，如 "Thunderstorm Advisory #VISPRSD"。
/// 这里只替换名称前缀，保留官方编号，方便和官网对照。
#[cfg(feature = "network")]
fn translate_alert_title(value: &str) -> String {
    const PREFIXES: [(&str, &str); 8] = [
        ("Thunderstorm Advisory", "雷暴提示"),
        ("Heavy Rainfall Warning", "强降雨警告"),
        ("Rainfall Advisory", "降雨提示"),
        ("Flood Advisory", "洪水提示"),
        ("Flood Warning", "洪水警告"),
        ("Gale Warning", "大风警告"),
        ("Tropical Cyclone", "台风提示"),
        ("Typhoon", "台风提示"),
    ];
    for (english, chinese) in PREFIXES {
        if value.len() >= english.len() && value[..english.len()].eq_ignore_ascii_case(english) {
            return format!("{chinese}{}", &value[english.len()..]);
        }
    }
    value.to_owned()
}

#[cfg(feature = "network")]
fn defense_for(kind: &str) -> &'static str {
    let lower = kind.to_ascii_lowercase();
    if lower.contains("rainfall") || lower.contains("flood") {
        "远离低洼地带和河道，留意官方疏散通知。"
    } else if lower.contains("thunderstorm") {
        "避免在空旷地带停留，收好阳台杂物。"
    } else if lower.contains("tropical cyclone") || lower.contains("typhoon") {
        "加固门窗，储备饮水和照明，关注路径更新。"
    } else if lower.contains("gale") || lower.contains("wind") {
        "减少海上和户外活动，固定易被吹落的物品。"
    } else {
        "以 PAGASA 官方公告和当地部门指引为准。"
    }
}

#[cfg(feature = "network")]
fn wind_direction_zh(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "N" => "北",
        "NNE" => "北东北",
        "NE" => "东北",
        "ENE" => "东东北",
        "E" => "东",
        "ESE" => "东东南",
        "SE" => "东南",
        "SSE" => "南东南",
        "S" => "南",
        "SSW" => "南西南",
        "SW" => "西南",
        "WSW" => "西西南",
        "W" => "西",
        "WNW" => "西西北",
        "NW" => "西北",
        "NNW" => "北西北",
        "" => "静风",
        other => other,
    }
    .to_owned()
}

#[cfg(feature = "network")]
fn hour_label(value: &str, first: bool) -> String {
    if first {
        return "现在".into();
    }
    let hour = value
        .split(':')
        .next()
        .and_then(|part| part.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let is_pm = value.to_ascii_uppercase().contains("PM");
    let hour = match (hour, is_pm) {
        (12, false) => 0,
        (12, true) => 12,
        (h, true) => h + 12,
        (h, false) => h,
    };
    format!("{hour}时")
}

#[cfg(feature = "network")]
pub(crate) fn translate_day(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "today" => "今天".into(),
        "tomorrow" => "明天".into(),
        "sun" | "sunday" => "周日".into(),
        "mon" | "monday" => "周一".into(),
        "tue" | "tuesday" => "周二".into(),
        "wed" | "wednesday" => "周三".into(),
        "thu" | "thursday" => "周四".into(),
        "fri" | "friday" => "周五".into(),
        "sat" | "saturday" => "周六".into(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_dashboard_has_five_days_and_a_place() {
        let dashboard = Dashboard::sample();
        assert_eq!(dashboard.forecast.len(), 5);
        assert_eq!(dashboard.hourly.len(), 24);
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

    #[test]
    fn aqi_categories_follow_the_six_bands() {
        assert_eq!(aqi_category(0), "优");
        assert_eq!(aqi_category(50), "优");
        assert_eq!(aqi_category(51), "良");
        assert_eq!(aqi_category(150), "轻度污染");
        assert_eq!(aqi_category(201), "重度污染");
        assert_eq!(aqi_category(400), "严重污染");
    }

    #[test]
    fn icons_are_derived_from_pagasa_descriptions() {
        assert_eq!(
            WeatherIcon::from_text(
                "Partly cloudy skies to at times cloudy with rainshowers and thunderstorms"
            ),
            WeatherIcon::Thunderstorm
        );
        assert_eq!(WeatherIcon::from_text("Clear skies"), WeatherIcon::Clear);
        assert_eq!(
            WeatherIcon::from_text("https://x/weather-icons/am/clear-skies.png"),
            WeatherIcon::Clear
        );
    }

    #[test]
    fn life_indices_react_to_rain_and_heat() {
        let mut current = Dashboard::sample().current;
        current.precip_mm = 8.0;
        current.feels_like_c = 45.0;
        let indices = life_indices(&current);
        assert_eq!(indices.len(), 4);
        assert_eq!(indices[1].level, "谨慎");
        assert_eq!(indices[2].level, "不宜");
    }

    #[cfg(feature = "network")]
    #[test]
    fn public_units_are_parsed_without_becoming_zero() {
        assert_eq!(parse_number("28 °C"), Some(28.0));
        assert_eq!(parse_number("72.8%"), Some(72.8));
        assert_eq!(parse_number("0.3 km/h"), Some(0.3));
        assert_eq!(parse_number("-1 °C"), Some(-1.0));
        assert_eq!(parse_number("- hPa"), None);
    }

    #[cfg(feature = "network")]
    #[test]
    fn hour_labels_use_24_hour_clock() {
        assert_eq!(hour_label("12:00 PM", false), "12时");
        assert_eq!(hour_label("1:00 PM", false), "13时");
        assert_eq!(hour_label("12:00 AM", false), "0时");
        assert_eq!(hour_label("9:00 AM", true), "现在");
    }
}
