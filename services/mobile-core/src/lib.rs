use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SnapshotMeta {
    #[serde(default)]
    pub source: Vec<String>,
    #[serde(default)]
    pub quality: String,
    #[serde(default)]
    pub fetched_at: String,
    #[serde(default)]
    pub cached: bool,
    #[serde(default)]
    pub stale: bool,
    #[serde(default)]
    pub request_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SnapshotError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NormalizedSnapshot {
    pub data: Value,
    pub meta: SnapshotMeta,
    #[serde(default)]
    pub errors: Vec<SnapshotError>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BackgroundProfile {
    pub mode: String,
    pub is_day: bool,
    pub primary_color: String,
    pub secondary_color: String,
    pub animation_intensity: f32,
    pub seed: u64,
    pub stale: bool,
    pub quality: String,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SnapshotErrorKind {
    #[error("snapshot JSON is invalid: {0}")]
    InvalidJson(String),
    #[error("snapshot envelope is missing a data object")]
    MissingData,
}

pub fn normalize_snapshot(json: &str) -> Result<NormalizedSnapshot, SnapshotErrorKind> {
    let value: Value = serde_json::from_str(json)
        .map_err(|error| SnapshotErrorKind::InvalidJson(error.to_string()))?;
    let data = value
        .get("data")
        .filter(|value| value.is_object())
        .cloned()
        .ok_or(SnapshotErrorKind::MissingData)?;
    let meta = value
        .get("meta")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let meta: SnapshotMeta = serde_json::from_value(meta)
        .map_err(|error| SnapshotErrorKind::InvalidJson(error.to_string()))?;
    let errors = value
        .get("errors")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    let errors: Vec<SnapshotError> = serde_json::from_value(errors)
        .map_err(|error| SnapshotErrorKind::InvalidJson(error.to_string()))?;
    Ok(NormalizedSnapshot { data, meta, errors })
}

pub fn background_profile(snapshot: &NormalizedSnapshot) -> BackgroundProfile {
    let current = snapshot
        .data
        .get("current")
        .or_else(|| snapshot.data.get("weather"))
        .unwrap_or(&Value::Null);
    let condition = string_field(current, &["condition", "summary", "desc", "weather"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let icon = string_field(current, &["icon", "weatherCode", "weather_code"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_day = current
        .get("isDay")
        .or_else(|| current.get("is_day"))
        .and_then(Value::as_bool)
        .or_else(|| snapshot.data.get("isDay").and_then(Value::as_bool))
        .unwrap_or(true);
    let mode = weather_mode(&condition, &icon, is_day);
    let quality = if snapshot.meta.quality.is_empty() {
        "unavailable".to_owned()
    } else {
        snapshot.meta.quality.clone()
    };
    let stale = snapshot.meta.stale;
    let (primary_color, secondary_color, base_intensity) = colors_for(&mode);
    let aqi = find_aqi(&snapshot.data).unwrap_or(0.0);
    let pollution_boost = if aqi > 100.0 { 0.1 } else { 0.0 };
    let animation_intensity = if stale {
        0.12
    } else {
        (base_intensity + pollution_boost).min(1.0)
    };
    BackgroundProfile {
        mode,
        is_day,
        primary_color: primary_color.to_owned(),
        secondary_color: secondary_color.to_owned(),
        animation_intensity,
        seed: stable_seed(snapshot),
        stale,
        quality,
    }
}

fn weather_mode(condition: &str, icon: &str, is_day: bool) -> String {
    let text = format!("{condition} {icon}");
    if text.contains("thunder") || text.contains("storm") || text.contains("雷") {
        "storm".to_owned()
    } else if text.contains("rain") || text.contains("shower") || text.contains("雨") {
        "rain".to_owned()
    } else if text.contains("fog")
        || text.contains("mist")
        || text.contains("haze")
        || text.contains("雾")
    {
        "fog".to_owned()
    } else if !is_day {
        "night".to_owned()
    } else if text.contains("cloud")
        || text.contains("overcast")
        || text.contains("多云")
        || text.contains("阴")
    {
        "cloudy".to_owned()
    } else if text.contains("clear") || text.contains("sun") || text.contains("晴") {
        "clear".to_owned()
    } else {
        "unknown".to_owned()
    }
}

fn colors_for(mode: &str) -> (&'static str, &'static str, f32) {
    match mode {
        "clear" => ("#2A9FE8", "#75D8FF", 0.35),
        "cloudy" => ("#667986", "#B7C9D1", 0.25),
        "rain" => ("#365D79", "#7CA6B5", 0.55),
        "storm" => ("#263C5A", "#6C7393", 0.70),
        "fog" => ("#5A646A", "#B1B7B4", 0.18),
        "night" => ("#1D2655", "#465A9B", 0.25),
        _ => ("#263238", "#607D8B", 0.10),
    }
}

fn string_field(value: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        value.get(*field).and_then(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .or_else(|| value.as_i64().map(|number| number.to_string()))
        })
    })
}

fn find_aqi(data: &Value) -> Option<f64> {
    [
        data.get("aqi"),
        data.pointer("/current/aqi"),
        data.pointer("/summary/maxAqi"),
        data.pointer("/summary/avgAqi"),
    ]
    .into_iter()
    .flatten()
    .find_map(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
}

fn stable_seed(snapshot: &NormalizedSnapshot) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let identity = format!(
        "{}|{}|{}|{}",
        snapshot.meta.source.join(","),
        snapshot.meta.fetched_at,
        snapshot.meta.request_id,
        snapshot
            .data
            .pointer("/location/latitude")
            .unwrap_or(&Value::Null)
    );
    for byte in identity.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(feature = "jni")]
mod jni_bridge {
    use super::{background_profile, normalize_snapshot};
    use jni::errors::ThrowRuntimeExAndDefault;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::EnvUnowned;

    #[no_mangle]
    pub extern "system" fn Java_org_panahon_zh_nativebridge_NativeBridge_nativeNormalizeSnapshot(
        mut env: EnvUnowned<'_>,
        _class: JClass<'_>,
        input: JString<'_>,
    ) -> jstring {
        env.with_env(|env| -> jni::errors::Result<_> {
            let input = input.mutf8_chars(env)?.to_str().into_owned();
            let output = match normalize_snapshot(&input) {
                Ok(snapshot) => serde_json::to_string(&snapshot).unwrap_or_default(),
                Err(error) => serde_json::json!({ "error": error.to_string() }).to_string(),
            };
            env.new_string(output)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
        .into_raw()
    }

    #[no_mangle]
    pub extern "system" fn Java_org_panahon_zh_nativebridge_NativeBridge_nativeBackgroundProfile(
        mut env: EnvUnowned<'_>,
        _class: JClass<'_>,
        input: JString<'_>,
    ) -> jstring {
        env.with_env(|env| -> jni::errors::Result<_> {
            let input = input.mutf8_chars(env)?.to_str().into_owned();
            let output = match normalize_snapshot(&input) {
                Ok(snapshot) => {
                    serde_json::to_string(&background_profile(&snapshot)).unwrap_or_default()
                }
                Err(error) => serde_json::json!({ "error": error.to_string() }).to_string(),
            };
            env.new_string(output)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
        .into_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn snapshot(current: Value, stale: bool) -> NormalizedSnapshot {
        NormalizedSnapshot {
            data: json!({ "location": { "latitude": 14.6 }, "current": current }),
            meta: SnapshotMeta {
                source: vec!["PAGASA".into()],
                quality: "official".into(),
                fetched_at: "2026-11-18T00:00:00Z".into(),
                cached: stale,
                stale,
                request_id: "test".into(),
            },
            errors: Vec::new(),
        }
    }

    #[test]
    fn normalizes_envelope_and_preserves_errors() {
        let value = normalize_snapshot(r#"{"data":{"current":{"condition":"晴"}},"meta":{"quality":"official"},"errors":[{"code":"notice","message":"cached"}]}"#).unwrap();
        assert_eq!(value.data["current"]["condition"], "晴");
        assert_eq!(value.errors[0].code, "notice");
    }

    #[test]
    fn derives_rain_profile_and_is_deterministic() {
        let snapshot = snapshot(json!({ "condition": "Rain showers", "isDay": true }), false);
        let first = background_profile(&snapshot);
        let second = background_profile(&snapshot);
        assert_eq!(first.mode, "rain");
        assert_eq!(first.primary_color, "#365D79");
        assert_eq!(first.seed, second.seed);
    }

    #[test]
    fn stale_snapshot_reduces_animation_and_night_wins_over_clear() {
        let snapshot = snapshot(json!({ "condition": "Clear", "isDay": false }), true);
        let profile = background_profile(&snapshot);
        assert_eq!(profile.mode, "night");
        assert_eq!(profile.animation_intensity, 0.12);
        assert!(profile.stale);
    }

    #[test]
    fn invalid_or_missing_data_is_rejected() {
        assert!(matches!(
            normalize_snapshot("not json"),
            Err(SnapshotErrorKind::InvalidJson(_))
        ));
        assert_eq!(
            normalize_snapshot(r#"{"meta":{}}"#),
            Err(SnapshotErrorKind::MissingData)
        );
    }
}

