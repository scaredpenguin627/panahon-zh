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
pub mod advisory;
pub mod emb;
pub mod trust;

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
        let text = text
            .split('?')
            .next()
            .unwrap_or(text)
            .rsplit('/')
            .next()
            .unwrap_or(text)
            .to_ascii_lowercase()
            .replace(['-', '_'], " ");
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
            Self::Rain => "雨",
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
    pub summary: Option<String>,
    pub track: Vec<MapTrackPoint>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MapTrackPoint {
    pub at: String,
    pub latitude: f32,
    pub longitude: f32,
    pub kind: String,
    pub radius_km: Option<f32>,
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
    /// Provenance of `temperature_c`. When this is `Unavailable` the UI must
    /// render a dash instead of the number.
    pub temperature_quality: DataQuality,
    pub feels_like_c: f32,
    pub humidity_percent: u8,
    /// Provenance of `humidity_percent`.
    pub humidity_quality: DataQuality,
    pub wind_kph: f32,
    /// Provenance of `wind_kph`.
    pub wind_quality: DataQuality,
    pub wind_direction: String,
    pub condition: String,
    pub updated_at: String,
    pub icon: WeatherIcon,
    pub is_day: bool,
    /// Station precipitation only; use precip_unit for the source's actual unit.
    pub precip_mm: f32,
    #[serde(default)]
    pub precip_unit: String,
    pub precip_desc: String,
    /// Provenance of the precipitation value shown in the UI.
    pub precip_quality: DataQuality,
    /// PAGASA heat-index band, e.g. "Caution". Empty when unknown.
    pub heat_index_desc: String,
    /// Provenance of the feels-like / heat-index value shown in the UI.
    pub heat_index_quality: DataQuality,
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
    pub low_c: Option<f32>,
    pub high_c: Option<f32>,
    pub precip_mm: f32,
    /// Provenance of the forecast precipitation value.
    pub precip_quality: DataQuality,
    pub icon: WeatherIcon,
    pub heat_index_c: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HourlyForecast {
    /// Already-localised hour label: 现在、13时 …
    pub time: String,
    pub summary: String,
    pub temperature_c: Option<f32>,
    pub precip_mm: f32,
    /// Provenance of the hourly precipitation value.
    pub precip_quality: DataQuality,
    pub icon: WeatherIcon,
    pub is_day: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub title_en: String,
    /// PAGASA alert type, e.g. "Rainfall Advisory".
    pub kind: String,
    pub area: String,
    pub issued_at: String,
    pub valid_until: String,
    pub detail: String,
    /// Template-based Chinese rendering of `detail`, when one could be built.
    /// The original English text always stays available.
    pub detail_zh: Option<String>,
    pub defense: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirQualityStation {
    #[serde(rename = "nameZh")]
    pub name_zh: String,
    /// 官方报表里的拉丁站名，英文界面使用。
    #[serde(default)]
    pub name: Option<String>,
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
    #[serde(default)]
    pub trend: Vec<AirQualityTrendPoint>,
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
pub struct AirQualityTrendPoint {
    pub at: String,
    pub aqi: Option<u16>,
    pub quality: DataQuality,
}

/// Model-backed sun and visibility details. These values are supplemental to
/// the PAGASA dashboard and are always shown with their forecast provenance.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SunWeather {
    pub uv_index: Option<f32>,
    pub visibility_km: Option<f32>,
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
    pub source: String,
}

/// A locally derived suggestion. No extra API call is involved.
#[derive(Clone, Debug, PartialEq)]
pub struct LifeIndex {
    pub title: String,
    pub level: String,
    pub advice: String,
    /// The observation value this suggestion was computed from, shown to the
    /// reader so the reasoning stays visible.
    pub basis: String,
    /// Current life indices are local heuristics unless an upstream product is
    /// added in the future.
    pub quality: DataQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dashboard {
    pub place: Place,
    pub current: CurrentWeather,
    pub forecast: Vec<ForecastDay>,
    pub hourly: Vec<HourlyForecast>,
    pub alerts: Vec<Alert>,
    pub sun: SunWeather,
    pub outlook_url: Option<String>,
    pub source: DataSource,
    #[serde(default)]
    pub fetched_at: i64,
    #[serde(default)]
    pub alerts_available: bool,
    #[serde(default)]
    pub notices: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    Unavailable,
    OfflineSample,
    PublicApi,
}

/// Describes where a displayed weather value came from.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataQuality {
    /// Supplied by an official observation or forecast field.
    Official,
    /// Supplied by an official forecast or nowcast field.
    Forecast,
    /// Derived locally from the latest observation and intended only as a
    /// quick reference.
    Estimated,
    /// The source did not provide a usable value.
    Unavailable,
}

impl DataQuality {
    /// True for values that came from an official PAGASA product.
    pub fn is_official(self) -> bool {
        matches!(self, Self::Official | Self::Forecast)
    }

    /// True when the value is meaningful enough to show.
    pub fn is_available(self) -> bool {
        !matches!(self, Self::Unavailable)
    }
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
    /// Empty production state. Samples are only entered explicitly in demo mode.
    pub fn empty(place: Place) -> Self {
        let mut data = Self::sample();
        data.place = place;
        data.source = DataSource::Unavailable;
        data.current.temperature_quality = DataQuality::Unavailable;
        data.current.humidity_quality = DataQuality::Unavailable;
        data.current.wind_quality = DataQuality::Unavailable;
        data.current.heat_index_quality = DataQuality::Unavailable;
        data.current.precip_quality = DataQuality::Unavailable;
        data.current.condition = "暂无数据".into();
        data.current.updated_at.clear();
        data.current.icon = WeatherIcon::Unknown;
        data.current.station = None;
        data.current.pressure_hpa = None;
        data.current.solar_wm2 = None;
        data.current.heat_index_desc.clear();
        data.forecast.clear();
        data.hourly.clear();
        data.alerts.clear();
        data.sun = SunWeather::default();
        data.alerts_available = false;
        data
    }
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
                temperature_quality: DataQuality::Estimated,
                feels_like_c: 34.0,
                humidity_percent: 72,
                humidity_quality: DataQuality::Estimated,
                wind_kph: 11.0,
                wind_quality: DataQuality::Estimated,
                wind_direction: "西南".into(),
                condition: "局部多云".into(),
                updated_at: "离线示例".into(),
                icon: WeatherIcon::PartlyCloudy,
                is_day: true,
                precip_mm: 0.0,
                precip_unit: "mm/h".into(),
                precip_desc: "无降雨".into(),
                precip_quality: DataQuality::Estimated,
                heat_index_desc: "注意".into(),
                heat_index_quality: DataQuality::Estimated,
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
                        temperature_c: Some(if afternoon { 32.0 } else { 27.0 }),
                        precip_mm: if afternoon { 0.2 } else { 0.0 },
                        precip_quality: DataQuality::Estimated,
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
                title_en: "Demo: heavy rain advisory".into(),
                kind: "Rainfall Advisory".into(),
                area: "马尼拉大都会".into(),
                issued_at: "离线示例".into(),
                valid_until: "离线示例".into(),
                detail: "联网后将从 PAGASA 公开接口加载官方预警。".into(),
                detail_zh: None,
                defense: "联网并确认数据来源后，再按官方防御指南行动。".into(),
            }],
            sun: SunWeather {
                uv_index: Some(5.0),
                visibility_km: Some(10.0),
                sunrise: Some("05:42".into()),
                sunset: Some("17:32".into()),
                source: "离线示例".into(),
            },
            outlook_url: None,
            source: DataSource::OfflineSample,
            fetched_at: 0,
            alerts_available: false,
            notices: Vec::new(),
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
        low_c: Some(low_c),
        high_c: Some(high_c),
        precip_mm,
        precip_quality: DataQuality::Estimated,
        icon,
        heat_index_c,
    }
}

/// 本地生活建议。
///
/// 每条建议都带上计算依据（`basis`），界面必须把依据一起显示出来。输入缺失时
/// 直接给"未知"，不要用默认值凑一个结论。
pub fn life_indices(current: &CurrentWeather) -> Vec<LifeIndex> {
    let temp = current.temperature_quality.is_available();
    let heat = current.heat_index_quality.is_official();
    vec![
        LifeIndex {
            title: "穿衣".into(),
            level: if temp { "参考气温" } else { "未知" }.into(),
            advice: "结合个人耐受、日照和风选择衣物；当前气温不能代表全天。".into(),
            basis: if temp {
                format!("气温 {:.0}°C", current.temperature_c)
            } else {
                "无数据".into()
            },
            quality: DataQuality::Unavailable,
        },
        LifeIndex {
            title: "热指数".into(),
            level: if heat {
                trust::heat_band(current.feels_like_c)
            } else {
                "未知"
            }
            .into(),
            advice: "PAGASA 热指数分级；日照和活动强度会改变个人风险。".into(),
            basis: if heat {
                format!("热指数 {:.0}°C", current.feels_like_c)
            } else {
                "无数据".into()
            },
            quality: if heat {
                current.heat_index_quality
            } else {
                DataQuality::Unavailable
            },
        },
        LifeIndex {
            title: "出行".into(),
            level: "查看预报".into(),
            advice: "查看出行时段的逐小时预报和预警；当前无雨不代表之后无雨。".into(),
            basis: "无官方指数".into(),
            quality: DataQuality::Unavailable,
        },
        LifeIndex {
            title: "晾晒".into(),
            level: "暂无指数".into(),
            advice: "缺少未来降雨、湿度、风和日照的完整指标，无法判断晾晒条件。".into(),
            basis: "无官方指数".into(),
            quality: DataQuality::Unavailable,
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
    // The connected EMB report covers NCR. Its maximum is not the AQI of an
    // arbitrary Philippine city; outside this area use the labelled CAMS model.
    if !(14.35..=14.95).contains(&latitude) || !(120.75..=121.35).contains(&longitude) {
        return fetch_cams_air_quality(latitude, longitude);
    }
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
        Ok(mut data) => {
            data.trend = fetch_air_quality_trend(latitude, longitude).unwrap_or_default();
            Ok(data)
        }
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
        name: Some("Current location (model)".into()),
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
        trend: fetch_air_quality_trend(latitude, longitude).unwrap_or_default(),
    })
}

#[cfg(feature = "network")]
fn fetch_air_quality_trend(
    latitude: f64,
    longitude: f64,
) -> Result<Vec<AirQualityTrendPoint>, String> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={latitude:.4}&longitude={longitude:.4}&hourly=us_aqi&past_days=1&forecast_days=1&timezone=auto"
    );
    let value = get_json(&url)?;
    let times = value
        .pointer("/hourly/time")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "空气质量趋势缺少时间序列".to_owned())?;
    let aqis = value
        .pointer("/hourly/us_aqi")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "空气质量趋势缺少 AQI 序列".to_owned())?;
    let start = times.len().saturating_sub(24);
    let mut trend = Vec::new();
    for index in start..times.len() {
        let at = times
            .get(index)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let aqi = aqis
            .get(index)
            .and_then(serde_json::Value::as_f64)
            .map(|value| value.round().clamp(0.0, 500.0) as u16);
        trend.push(AirQualityTrendPoint {
            at,
            aqi,
            quality: DataQuality::Forecast,
        });
    }
    Ok(trend)
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
                summary: None,
                track: Vec::new(),
            })
        }
        "typhoon" => {
            let value = map_json(&format!("{MAP_ROOT}/api/v1/cyclone-track?token={token}"))?;
            let cyclone = value
                .as_array()
                .and_then(|rows| rows.first())
                .unwrap_or(&value);
            let info = cyclone
                .pointer("/info")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| "台风图层响应格式无法解析".to_owned())?;
            let mut track = info
                .iter()
                .filter_map(|(at, row)| {
                    Some(MapTrackPoint {
                        at: at.to_owned(),
                        latitude: row.get("latitude")?.as_str()?.parse().ok()?,
                        longitude: row.get("longitude")?.as_str()?.parse().ok()?,
                        kind: row
                            .get("cyclone_type")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("?")
                            .to_owned(),
                        radius_km: row
                            .get("radius")
                            .and_then(serde_json::Value::as_str)
                            .and_then(|value| value.parse().ok()),
                    })
                })
                .collect::<Vec<_>>();
            track.sort_by(|a, b| a.at.cmp(&b.at));
            let latest = track.last();
            let summary = latest.map(|row| {
                let latitude = row.latitude;
                let longitude = row.longitude;
                let kind = row.kind.as_str();
                let radius = row.radius_km.unwrap_or(0.0);
                let name = cyclone
                    .get("cyclone_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("未命名台风");
                Some(format!(
                    "Cyclone {name} · {kind} · {latitude:.1}°N {longitude:.1}°E · radius {radius:.0} km · {}",
                    row.at
                ))
            });
            Ok(RemoteMapSnapshot {
                layer: layer.to_owned(),
                latest_at: latest.map(|point| point.at.clone()),
                frame_count: None,
                item_count: Some(info.len() as u32),
                image: None,
                summary: summary.flatten(),
                track,
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
                summary: None,
                track: Vec::new(),
            })
        }
        _ => Err(format!("未知地图图层：{layer}")),
    }
}

/// Fetch a specific frame from a time-based map layer. `index` is zero-based;
/// out-of-range values are clamped to the newest available frame. The latest
/// frame remains the default path used by [`fetch_remote_map_layer`].
#[cfg(feature = "network")]
pub fn fetch_remote_map_layer_frame(
    layer: &str,
    index: usize,
) -> Result<RemoteMapSnapshot, String> {
    let token = fetch_map_token()?;
    match layer {
        "radar" => fetch_radar_snapshot_at(&token, Some(index)),
        "satellite" => fetch_satellite_snapshot_at(&token, Some(index)),
        _ => fetch_remote_map_layer(layer),
    }
}

#[cfg(not(feature = "network"))]
pub fn fetch_remote_map_layer(_layer: &str) -> Result<RemoteMapSnapshot, String> {
    Err("网络适配器未启用".into())
}

#[cfg(not(feature = "network"))]
pub fn fetch_remote_map_layer_frame(
    _layer: &str,
    _index: usize,
) -> Result<RemoteMapSnapshot, String> {
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
    fetch_radar_snapshot_at(token, None)
}

#[cfg(feature = "network")]
fn fetch_radar_snapshot_at(
    token: &str,
    frame_index: Option<usize>,
) -> Result<RemoteMapSnapshot, String> {
    let sublayer = "mosaic-rainrate";
    let value = map_json(&format!(
        "{MAP_ROOT}/api/v1/radar/timeline?token={token}&sublayer={sublayer}"
    ))?;
    let rows = value
        .pointer("/data/timeline")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "雷达图层响应格式无法解析".to_owned())?;
    let selected = rows
        .get(
            frame_index
                .unwrap_or_else(|| rows.len().saturating_sub(1))
                .min(rows.len().saturating_sub(1)),
        )
        .ok_or_else(|| "雷达图层没有可用时间帧".to_owned())?;
    let timestamp = selected
        .get("observed_at_unix")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| "雷达图层缺少时间戳".to_owned())?;
    let observed_at = selected
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
        summary: None,
        track: Vec::new(),
    })
}

#[cfg(feature = "network")]
fn fetch_satellite_snapshot(token: &str) -> Result<RemoteMapSnapshot, String> {
    fetch_satellite_snapshot_at(token, None)
}

#[cfg(feature = "network")]
fn fetch_satellite_snapshot_at(
    token: &str,
    frame_index: Option<usize>,
) -> Result<RemoteMapSnapshot, String> {
    let value = map_json(&format!(
        "{MAP_ROOT}/api/v1/himawari/timeline?token={token}"
    ))?;
    let rows = value
        .pointer("/data/timeline")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "卫星图层响应格式无法解析".to_owned())?;
    let index = frame_index
        .unwrap_or_else(|| rows.len().saturating_sub(1))
        .min(rows.len().saturating_sub(1));
    let latest = rows
        .get(index)
        .ok_or_else(|| "卫星图层没有可用时间帧".to_owned())?;
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
        summary: None,
        track: Vec::new(),
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

/// 从自动站列表里挑出与地点对应的观测。
///
/// PAGASA 的 `/observation/automatic` 不看经纬度，任何坐标都返回同一批站点。
/// 这里按站名匹配城市或所在的都市圈；匹配不到就返回 `None`，让调用方退回官方预报。
#[cfg(feature = "network")]
fn pick_station<'a>(
    observation: &'a serde_json::Value,
    place: &Place,
) -> Option<&'a serde_json::Value> {
    let rows = observation.pointer("/data")?.as_array()?;
    let city = place_key(&place.name);
    let region = place_key(&place.region);
    let metro = metro_tokens(&place.region);

    let mut best: Option<(u32, &serde_json::Value)> = None;
    for row in rows {
        let Some(obs) = row.get("obs") else {
            continue;
        };
        let Some(loc) = obs.get("loc").and_then(|value| value.as_str()) else {
            continue;
        };
        let key = station_key(loc);

        let mut score = 0u32;
        if !city.is_empty() && key.split_whitespace().any(|word| word == city.as_str()) {
            score = 100;
        } else if !city.is_empty() && !key.is_empty() && key.contains(&city) {
            score = 80;
        } else if !region.is_empty() && !key.is_empty() && key.contains(&region) {
            score = 60;
        } else if metro.iter().any(|token| key.contains(token)) {
            score = 50;
        }

        if score > 0 && best.is_none_or(|(best_score, _)| score > best_score) {
            best = Some((score, obs));
        }
    }
    best.map(|(_, obs)| obs)
}

/// 站名归一化：小写、去标点、压空格。保留 "city" 等词，供短语匹配使用。
#[cfg(feature = "network")]
fn station_key(value: &str) -> String {
    let lowered = value.to_ascii_lowercase();
    let cleaned: String = lowered
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 地点名归一化：去掉末尾的 "city"，"Cebu City" 和 "Mactan, Cebu" 才能对上。
#[cfg(feature = "network")]
fn place_key(value: &str) -> String {
    let key = station_key(value);
    key.strip_suffix(" city").map(str::to_owned).unwrap_or(key)
}

/// 马尼拉大都会的自动站分布在各个市，单独给一组关键词。
#[cfg(feature = "network")]
fn metro_tokens(region: &str) -> Vec<&'static str> {
    let region = region.to_ascii_lowercase();
    if !region.contains("manila") {
        return Vec::new();
    }
    // 必须用短语："quezon" 会同时命中 Quezon 省，而省站的观测往往更旧
    vec![
        "quezon city",
        "city of manila",
        "manila",
        "makati",
        "pasay",
        "caloocan",
        "pasig",
        "taguig",
        "marikina",
        "muntinlupa",
        "paranaque",
        "valenzuela",
        "navotas",
        "malabon",
        "mandaluyong",
        "san juan",
        "las pinas",
        "pateros",
    ]
}
#[cfg(feature = "network")]
pub fn fetch_public_dashboard(place: &Place) -> Result<Dashboard, String> {
    let query = format!(
        "latitude={:.5}&longitude={:.5}",
        place.latitude, place.longitude
    );
    let nowcast = get_json(&format!("{API_ROOT}/nowcast/dashboard?{query}"))?;
    let observation_result = get_json(&format!("{API_ROOT}/observation/automatic?{query}"));
    let alerts_result = get_json(&format!("{API_ROOT}/alerts"));
    let observation = observation_result.clone().unwrap_or_default();
    let alerts = alerts_result.clone().unwrap_or_default();
    let mut dashboard =
        decode_dashboard(place, &nowcast, &observation, &alerts, trust::now_epoch())?;
    if observation_result.is_err() {
        dashboard.notices.push("observation_unavailable".into());
    }
    if alerts_result.is_err() || !alerts.pointer("/data").is_some_and(|v| v.is_array()) {
        dashboard.alerts_available = false;
        dashboard.notices.push("alerts_unavailable".into());
    }
    dashboard.sun = fetch_sun_weather(place).unwrap_or_default();
    Ok(dashboard)
}

#[cfg(feature = "network")]
pub fn decode_dashboard(
    place: &Place,
    nowcast: &serde_json::Value,
    observation: &serde_json::Value,
    alerts: &serde_json::Value,
    fetched_at: i64,
) -> Result<Dashboard, String> {
    if !nowcast.pointer("/data/nc").is_some_and(|v| v.is_object()) {
        return Err("天气响应缺少有效的预报数据".into());
    }
    let now = nowcast
        .pointer("/data/nc")
        .or_else(|| nowcast.pointer("/data"));
    // 观测接口忽略坐标，固定返回全国 112 个站，必须按地名自己挑
    let obs = pick_station(observation, place).filter(|obs| {
        text(Some(obs), "observed_at").is_some_and(|at| trust::fresh_observation(&at, fetched_at))
    });

    let icon_source = text(obs, "icon").or_else(|| text(now, "icon"));
    let description = text(obs, "desc").or_else(|| text(now, "desc"));
    let icon = weather_icon(description.as_deref(), icon_source.as_deref());

    let (temperature, temperature_quality) = if let Some(value) = bounded(obs, "temp", -70.0, 60.0)
    {
        (value as f32, DataQuality::Official)
    } else if let Some(value) = bounded(now, "tmp", -70.0, 60.0) {
        (value as f32, DataQuality::Forecast)
    } else {
        (0.0, DataQuality::Unavailable)
    };
    let (humidity, humidity_quality) = match bounded(obs, "hum", 0.0, 100.0) {
        Some(value) => (value.clamp(0.0, 100.0) as u8, DataQuality::Official),
        None => (0, DataQuality::Unavailable),
    };
    let (wind, wind_quality) = match bounded(obs, "ws", 0.0, 450.0) {
        Some(value) => (value as f32, DataQuality::Official),
        None => (0.0, DataQuality::Unavailable),
    };
    let (feels_like, heat_index_quality) =
        if let Some(value) = bounded(obs, "heat_index", -70.0, 90.0) {
            (value as f32, DataQuality::Official)
        } else if let Some(value) = bounded(now, "hindex", -70.0, 90.0) {
            (value as f32, DataQuality::Forecast)
        } else {
            (temperature, DataQuality::Unavailable)
        };

    let (precip_mm, precip_quality) = if let Some(value) = bounded(obs, "precip_1h", 0.0, 2000.0) {
        (value.max(0.0) as f32, DataQuality::Official)
    } else {
        (0.0, DataQuality::Unavailable)
    };

    let current = CurrentWeather {
        temperature_c: temperature,
        temperature_quality,
        feels_like_c: feels_like,
        humidity_percent: humidity,
        humidity_quality,
        wind_kph: wind,
        wind_quality,
        wind_direction: text(obs, "wd")
            .map(|value| wind_direction_zh(&value))
            .unwrap_or_else(|| "未知".into()),
        condition: description
            .clone()
            .map(translate_condition)
            .unwrap_or_else(|| "暂无描述".into()),
        updated_at: text(obs, "observed_at")
            .or_else(|| text(now, "time"))
            .unwrap_or_else(|| "公开接口".into()),
        icon,
        is_day: icon_source.as_deref().map(is_day_icon).unwrap_or(true),
        precip_mm,
        precip_unit: if text(obs, "precip_1h").is_some_and(|v| v.contains("/h")) {
            "mm/h"
        } else {
            "mm"
        }
        .into(),
        precip_desc: if !precip_quality.is_available() {
            "暂无降水数据".into()
        } else if precip_mm <= 0.05 {
            "无降雨".into()
        } else {
            format!("约 {precip_mm:.1} mm/h")
        },
        precip_quality,
        heat_index_desc: if heat_index_quality.is_available() {
            trust::heat_band(feels_like).into()
        } else {
            String::new()
        },
        heat_index_quality,
        pressure_hpa: bounded(obs, "mslp", 800.0, 1100.0).map(|value| value as f32),
        solar_wm2: bounded(obs, "sr", 0.0, 1600.0).map(|value| value as f32),
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
                    let icon_url = text(Some(row), "icon");
                    let icon = weather_icon(Some(&description), icon_url.as_deref());
                    let precip = bounded(Some(row), "tprecip", 0.0, 3000.0);
                    let date_key = trust::forecast_date(
                        &text(Some(row), "d").unwrap_or_default(),
                        &text(Some(row), "dt").unwrap_or_default(),
                        chrono::DateTime::from_timestamp(fetched_at + 8 * 3600, 0)
                            .map(|value| value.date_naive())
                            .unwrap_or_else(trust::manila_today),
                    );
                    ForecastDay {
                        date: if date_key.is_empty() {
                            "未知".into()
                        } else {
                            trust::day_label(&date_key, false)
                        },
                        date_label: text(Some(row), "dt").unwrap_or_default(),
                        summary: translate_condition(description),
                        low_c: bounded(Some(row), "tmin", -70.0, 60.0).map(|v| v as f32),
                        high_c: bounded(Some(row), "tmax", -70.0, 60.0).map(|v| v as f32),
                        precip_mm: precip.unwrap_or(0.0) as f32,
                        precip_quality: if precip.is_some() {
                            DataQuality::Forecast
                        } else {
                            DataQuality::Unavailable
                        },
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
                .map(|row| {
                    let description = text(Some(row), "desc").unwrap_or_default();
                    let icon_url = text(Some(row), "icon");
                    let icon = weather_icon(Some(&description), icon_url.as_deref());
                    let precip = bounded(Some(row), "tprecip", 0.0, 2000.0);
                    HourlyForecast {
                        time: text(Some(row), "time")
                            .map(|value| trust::clock_label(&value))
                            .unwrap_or_else(|| "未知".into()),
                        summary: translate_condition(description),
                        temperature_c: bounded(Some(row), "tmp", -70.0, 60.0).map(|v| v as f32),
                        precip_mm: precip.unwrap_or(0.0) as f32,
                        precip_quality: if precip.is_some() {
                            DataQuality::Forecast
                        } else {
                            DataQuality::Unavailable
                        },
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
            let parsed = items
                .iter()
                .filter(|item| {
                    text(Some(item), "valid_until_at")
                        .and_then(|v| trust::timestamp(&v))
                        .is_none_or(|at| at > fetched_at)
                })
                .map(|item| {
                    let kind = text(Some(item), "type").unwrap_or_else(|| "天气预警".into());
                    let raw_title = text(Some(item), "title").unwrap_or_else(|| "天气预警".into());
                    let detail =
                        text(Some(item), "message").unwrap_or_else(|| "请查看官方详情".into());
                    Alert {
                        id: text(Some(item), "alert_id").unwrap_or_default(),
                        title: raw_title.clone(),
                        title_en: raw_title,
                        kind: kind.clone(),
                        area: text(Some(item), "locations").unwrap_or_else(|| "未注明区域".into()),
                        issued_at: text(Some(item), "issued_at")
                            .unwrap_or_else(|| "未知时间".into()),
                        valid_until: text(Some(item), "valid_until_at").unwrap_or_default(),
                        detail,
                        // Keep the official PAGASA message intact.  The UI
                        // offers a copy action so users can translate it in
                        // their own trusted app without mixed-language text.
                        detail_zh: None,
                        defense: String::new(),
                    }
                })
                .collect::<Vec<_>>();

            // PAGASA returns a global alert feed. Prefer alerts whose stated
            // area contains the selected city/province, while retaining
            // explicitly nationwide alerts. This avoids presenting unrelated
            // warnings from the rest of the country as local notifications.
            parsed
                .into_iter()
                .filter(|alert| {
                    alert_is_global(&alert.area) || alert_matches_place(&alert.area, place)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let sun = SunWeather::default();
    let outlook_url = nowcast
        .pointer("/data/public_weather_forecast/url")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);

    Ok(Dashboard {
        place: place.clone(),
        current,
        forecast,
        hourly: hourly_items,
        alerts: alert_items,
        sun,
        outlook_url,
        source: DataSource::PublicApi,
        fetched_at,
        alerts_available: alerts.pointer("/data").is_some_and(|v| v.is_array()),
        notices: if obs.is_none() {
            vec!["no_fresh_station".into()]
        } else {
            Vec::new()
        },
    })
}

#[cfg(feature = "network")]
fn alert_matches_place(area: &str, place: &Place) -> bool {
    let haystack = normalize_location(area);
    if haystack.is_empty() {
        return false;
    }

    let mut candidates = vec![
        place.name.as_str(),
        place.region.as_str(),
        place.id.as_str(),
    ];
    // The offline sample uses Chinese labels, while PAGASA's alert feed uses
    // the English regional aliases. Keep the common NCR aliases together so
    // the first online refresh still filters correctly.
    let ncr = normalize_location(&format!("{} {}", place.name, place.region));
    if place.id.eq_ignore_ascii_case("manila")
        || ncr.contains("manila")
        || ncr.contains("capital")
        || (place.latitude > 14.35
            && place.latitude < 14.95
            && place.longitude > 120.75
            && place.longitude < 121.35)
    {
        candidates.extend(["NCR", "Metro Manila", "National Capital Region"]);
    }

    candidates
        .into_iter()
        .map(normalize_location)
        .filter(|candidate| candidate.len() >= 3)
        .any(|candidate| haystack.contains(&candidate))
}

#[cfg(feature = "network")]
fn alert_is_global(area: &str) -> bool {
    let normalized = normalize_location(area);
    [
        "nationwide",
        "entirecountry",
        "allareas",
        "allregions",
        "philippines",
    ]
    .into_iter()
    .any(|marker| normalized.contains(marker))
}

#[cfg(feature = "network")]
fn normalize_location(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(not(feature = "network"))]
pub fn fetch_public_dashboard(_place: &Place) -> Result<Dashboard, String> {
    Err("网络适配器未启用".into())
}

#[cfg(feature = "network")]
pub fn search_places(query: &str) -> Result<Vec<Place>, String> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Err("请输入至少 2 个字符".into());
    }
    let api_query = english_location_alias(query).unwrap_or(query);
    let mut results = known_places(query);
    if !results.is_empty() {
        return Ok(results);
    }
    // The public endpoint is primarily indexed in English.  Keep the local
    // Chinese aliases above available even when the network is unavailable.
    // Unknown places still go through the official location endpoint.
    let encoded = percent_encode(api_query);
    let response = match get_json(&format!("{API_ROOT}/location/search?q={encoded}")) {
        Ok(value) => value,
        Err(_error) if !results.is_empty() => return Ok(results),
        Err(error) => return Err(error),
    };
    let rows = response
        .as_array()
        .or_else(|| {
            response
                .pointer("/data")
                .and_then(serde_json::Value::as_array)
        })
        .ok_or_else(|| "地点响应格式无法解析".to_owned())?;
    let remote = rows
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
        .collect::<Vec<_>>();
    for place in remote {
        if !results.iter().any(|existing| {
            (existing.latitude - place.latitude).abs() < 0.0001
                && (existing.longitude - place.longitude).abs() < 0.0001
        }) {
            results.push(place);
        }
    }
    Ok(results)
}

#[cfg(feature = "network")]
fn known_places(query: &str) -> Vec<Place> {
    let query = query.trim().to_ascii_lowercase();
    let entries: [(&str, &str, &str, f64, f64, &[&str]); 10] = [
        (
            "manila",
            "马尼拉",
            "国家首都区",
            14.5995,
            120.9842,
            &["manila", "metro manila", "ncr", "马尼拉", "马尼拉大都会"],
        ),
        (
            "cebu",
            "宿务",
            "中米沙鄢",
            10.3157,
            123.8854,
            &["cebu", "cebu city", "宿务", "宿务市"],
        ),
        (
            "davao",
            "达沃",
            "达沃区",
            7.1907,
            125.4553,
            &["davao", "davao city", "达沃", "达沃市"],
        ),
        (
            "baguio",
            "碧瑶",
            "科迪勒拉行政区",
            16.4023,
            120.5960,
            &["baguio", "碧瑶"],
        ),
        (
            "iloilo",
            "伊洛伊洛",
            "西米沙鄢",
            10.7202,
            122.5621,
            &["iloilo", "iloilo city", "伊洛伊洛"],
        ),
        (
            "bacolod",
            "巴科洛德",
            "西米沙鄢",
            10.6765,
            122.9509,
            &["bacolod", "巴科洛德"],
        ),
        (
            "cagayan-de-oro",
            "卡加延德奥罗",
            "北棉兰老",
            8.4542,
            124.6319,
            &["cagayan de oro", "cagayan", "卡加延德奥罗"],
        ),
        (
            "angeles",
            "安吉利斯",
            "中吕宋",
            15.1450,
            120.5887,
            &["angeles", "angeles city", "安吉利斯"],
        ),
        (
            "tagaytay",
            "大雅台",
            "甲米地省",
            14.1153,
            120.9621,
            &["tagaytay", "大雅台"],
        ),
        (
            "puerto-princesa",
            "公主港",
            "巴拉望省",
            9.7392,
            118.7353,
            &["puerto princesa", "puerto-princesa", "公主港"],
        ),
    ];
    entries
        .into_iter()
        .filter(|(_, _, _, _, _, aliases)| {
            aliases.iter().any(|alias| {
                let alias = alias.to_ascii_lowercase();
                alias.contains(&query) || query.contains(&alias)
            })
        })
        .map(|(id, name, region, latitude, longitude, _)| Place {
            id: id.into(),
            name: name.into(),
            region: region.into(),
            latitude,
            longitude,
        })
        .collect()
}

#[cfg(feature = "network")]
fn english_location_alias(query: &str) -> Option<&'static str> {
    let normalized = query.trim();
    match normalized {
        "马尼拉" | "马尼拉大都会" => Some("Manila"),
        "宿务" | "宿务市" => Some("Cebu City"),
        "达沃" | "达沃市" => Some("Davao City"),
        "碧瑶" => Some("Baguio"),
        "伊洛伊洛" => Some("Iloilo City"),
        "巴科洛德" => Some("Bacolod"),
        "卡加延德奥罗" => Some("Cagayan de Oro"),
        "安吉利斯" => Some("Angeles City"),
        "大雅台" => Some("Tagaytay"),
        "公主港" => Some("Puerto Princesa"),
        _ => None,
    }
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
fn fetch_sun_weather(place: &Place) -> Result<SunWeather, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={:.5}&longitude={:.5}&current=uv_index,visibility&daily=sunrise,sunset&timezone=auto&forecast_days=1",
        place.latitude, place.longitude
    );
    let value = get_json(&url)?;
    let uv_index = number(value.pointer("/current"), "uv_index").map(|value| value as f32);
    let visibility_km =
        number(value.pointer("/current"), "visibility").map(|value| (value / 1000.0) as f32);
    let sunrise = value
        .pointer("/daily/sunrise/0")
        .and_then(serde_json::Value::as_str)
        .map(short_iso_clock);
    let sunset = value
        .pointer("/daily/sunset/0")
        .and_then(serde_json::Value::as_str)
        .map(short_iso_clock);
    if uv_index.is_none() && visibility_km.is_none() && sunrise.is_none() && sunset.is_none() {
        return Err("Open-Meteo 没有返回日照补充数据".into());
    }
    Ok(SunWeather {
        uv_index,
        visibility_km,
        sunrise,
        sunset,
        source: "Open-Meteo 模型预报".into(),
    })
}

#[cfg(feature = "network")]
fn short_iso_clock(value: &str) -> String {
    value
        .get(11..16)
        .filter(|clock| clock.as_bytes().get(2) == Some(&b':'))
        .unwrap_or(value)
        .to_owned()
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
    trust::measurement(value)
}

#[cfg(feature = "network")]
fn bounded(value: Option<&serde_json::Value>, key: &str, low: f64, high: f64) -> Option<f64> {
    number(value, key).filter(|v| v.is_finite() && (low..=high).contains(v))
}

pub fn weather_icon(description: Option<&str>, filename: Option<&str>) -> WeatherIcon {
    let described = description
        .map(WeatherIcon::from_text)
        .unwrap_or(WeatherIcon::Unknown);
    if described != WeatherIcon::Unknown {
        described
    } else {
        filename
            .map(WeatherIcon::from_text)
            .unwrap_or(WeatherIcon::Unknown)
    }
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
    } else if lower.contains("heavy rain") {
        "大雨".into()
    } else if lower.contains("rainshower") || lower.contains("shower") {
        "阵雨".into()
    } else if lower.contains("rain") {
        "雨".into()
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
#[allow(dead_code)]
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
#[allow(dead_code)]
fn translate_heat_index(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("extreme") {
        if lower.contains("caution") {
            "注意防暑".into()
        } else {
            "极易中暑".into()
        }
    } else if lower.contains("danger") {
        "有中暑危险".into()
    } else if lower.contains("caution") {
        "注意补水".into()
    } else {
        value.to_owned()
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(crate) fn translate_day(value: String) -> String {
    let trimmed = value.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "today" => "今天".into(),
        "tomorrow" => "明天".into(),
        "sun" | "sunday" => "周日".into(),
        "mon" | "monday" => "周一".into(),
        "tue" | "tuesday" => "周二".into(),
        "wed" | "wednesday" => "周三".into(),
        "thu" | "thursday" => "周四".into(),
        "fri" | "friday" => "周五".into(),
        "sat" | "saturday" => "周六".into(),
        _ => {
            // Some API revisions return an ISO date in `d` instead of a
            // weekday. Keep the label compact and readable rather than
            // exposing a timestamp such as `2026-11-09T00:00:00Z`.
            let date = trimmed.split('T').next().unwrap_or(trimmed);
            let parts = date
                .split(|ch: char| ch == '-' || ch == '/' || ch == '.')
                .filter_map(|part| part.parse::<u32>().ok())
                .collect::<Vec<_>>();
            if parts.len() >= 2 {
                let (month, day) = if parts.len() >= 3 {
                    (parts[parts.len() - 2], parts[parts.len() - 1])
                } else {
                    (parts[0], parts[1])
                };
                if (1..=12).contains(&month) && (1..=31).contains(&day) {
                    return format!("{month}月{day}日");
                }
            }
            trimmed.to_owned()
        }
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
    fn sample_dashboard_has_sun_details_for_offline_rendering() {
        let dashboard = Dashboard::sample();
        assert!(dashboard.sun.sunrise.is_some());
        assert!(dashboard.sun.sunset.is_some());
        assert!(dashboard.sun.uv_index.is_some());
    }

    #[test]
    fn air_quality_without_trend_stays_backward_compatible() {
        let json = r#"{
            "source":{"provider":"test","page":"","note":""},
            "updatedAt":null,"dataClock":null,"fetchedAt":null,
            "summary":{"stationCount":0,"avgAqi":null,"maxAqi":null,"maxStation":null,"category":null},
            "stations":[]
        }"#;
        let air: AirQuality = serde_json::from_str(json).expect("legacy AQI payload");
        assert!(air.trend.is_empty());
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
    fn data_quality_distinguishes_official_forecast_from_estimates() {
        assert!(DataQuality::Official.is_official());
        assert!(DataQuality::Forecast.is_official());
        assert!(!DataQuality::Estimated.is_official());
        assert!(!DataQuality::Unavailable.is_official());
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

    #[cfg(feature = "network")]
    #[test]
    fn heavy_rain_descriptions_keep_their_severity() {
        assert_eq!(translate_condition("Heavy rain showers".into()), "大雨");
        assert_eq!(translate_condition("Rain showers".into()), "阵雨");
    }

    #[test]
    fn life_indices_react_to_rain_and_heat() {
        let mut current = Dashboard::sample().current;
        current.precip_mm = 8.0;
        current.feels_like_c = 45.0;
        let indices = life_indices(&current);
        assert_eq!(indices.len(), 4);
        assert_eq!(indices[1].level, "未知");
        assert_eq!(indices[2].level, "查看预报");
        assert!(indices
            .iter()
            .all(|index| index.quality == DataQuality::Unavailable));
    }

    #[test]
    fn missing_precipitation_is_not_presented_as_zero_rain() {
        let mut current = Dashboard::sample().current;
        current.precip_mm = 0.0;
        current.precip_quality = DataQuality::Unavailable;
        current.heat_index_quality = DataQuality::Unavailable;
        let indices = life_indices(&current);
        assert_eq!(indices[1].level, "未知");
        assert_eq!(indices[2].level, "查看预报");
        assert_eq!(indices[3].level, "暂无指数");
    }

    #[cfg(feature = "network")]
    #[test]
    fn station_matching_uses_the_place_name() {
        // "Alabat, Quezon" 属于 Quezon 省，不能因为含 "quezon" 就被选成马尼拉的站
        let observation = serde_json::json!({
            "data": [
                { "obs": { "loc": "Alabat, Quezon", "temp": "24.0°C" } },
                { "obs": { "loc": "Science Garden, Quezon City", "temp": "31.6°C" } },
                { "obs": { "loc": "Mactan, Cebu", "temp": "30.1°C" } },
                { "obs": { "loc": "Dalaguete, Cebu", "temp": "29.4°C" } }
            ]
        });
        let manila = Place {
            id: "manila".into(),
            name: "Manila".into(),
            region: "Metro Manila".into(),
            latitude: 14.5995,
            longitude: 120.9842,
        };
        let cebu = Place {
            id: "cebu".into(),
            name: "Cebu City".into(),
            region: "Central Visayas".into(),
            latitude: 10.3157,
            longitude: 123.8854,
        };
        let baguio = Place {
            id: "baguio".into(),
            name: "Baguio".into(),
            region: "Cordillera".into(),
            latitude: 16.4023,
            longitude: 120.5960,
        };

        let picked = pick_station(&observation, &manila).and_then(|obs| text(Some(obs), "loc"));
        assert_eq!(picked.as_deref(), Some("Science Garden, Quezon City"));

        let picked = pick_station(&observation, &cebu).and_then(|obs| text(Some(obs), "loc"));
        assert_eq!(picked.as_deref(), Some("Mactan, Cebu"));

        assert!(pick_station(&observation, &baguio).is_none());
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

    #[cfg(feature = "network")]
    #[test]
    fn alerts_are_scoped_to_the_selected_place_or_nationwide() {
        let manila = Place {
            id: "manila".into(),
            name: "马尼拉".into(),
            region: "国家首都区".into(),
            latitude: 14.5995,
            longitude: 120.9842,
        };
        assert!(alert_matches_place("Metro Manila", &manila));
        assert!(alert_matches_place("NCR", &manila));
        assert!(!alert_matches_place("Cebu Province", &manila));
        assert!(alert_is_global("Nationwide Philippines"));
    }

    #[cfg(feature = "network")]
    #[test]
    fn chinese_city_aliases_produce_selectable_places_without_network() {
        let cebu = known_places("宿务");
        assert_eq!(cebu.len(), 1);
        assert_eq!(cebu[0].id, "cebu");
        assert_eq!(cebu[0].name, "宿务");

        let manila = known_places("Manila");
        assert_eq!(manila.len(), 1);
        assert_eq!(manila[0].id, "manila");
    }

    #[cfg(feature = "network")]
    #[test]
    fn heat_index_translation_keeps_extreme_caution_distinct() {
        assert_eq!(translate_heat_index("Extreme Caution"), "注意防暑");
        assert_eq!(translate_heat_index("Danger"), "有中暑危险");
        assert_eq!(translate_heat_index("Extreme Danger"), "极易中暑");
    }

    #[cfg(feature = "network")]
    #[test]
    fn date_labels_are_compact_for_iso_dates() {
        assert_eq!(translate_day("2026-11-09".into()), "11月9日");
        assert_eq!(translate_day("11/09".into()), "11月9日");
        assert_eq!(translate_day("today".into()), "今天");
    }
}
