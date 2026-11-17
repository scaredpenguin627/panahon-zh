use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_PAGASA_API: &str = "https://api.mobile.panahon.gov.ph/api/v1";
const DEFAULT_PAGASA_MAP: &str = "https://origin.panahon.gov.ph";
const DEFAULT_OPEN_METEO: &str = "https://api.open-meteo.com/v1/forecast";
const DEFAULT_OPEN_METEO_AIR: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";
const DEFAULT_TIMEOUT_MS: u64 = 15_000;
const EMB_RESOURCE_KEY: &str = "156a380e-4084-4645-8a01-6a1e845d7416";
const EMB_TENANT_ID: &str = "f6f4a692-43b3-433b-92b2-65c4e6ccd920";
const EMB_POWERBI_APP: &str = "https://app.powerbi.com";

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct Config {
    pub bind: String,
    pub pagasa_api: String,
    pub pagasa_map: String,
    pub open_meteo: String,
    pub open_meteo_air: String,
    pub emb_aqi: Option<String>,
    pub timeout: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind: env_or("PANAHON_BIND", "0.0.0.0:8788"),
            pagasa_api: env_or("PAGASA_API_ROOT", DEFAULT_PAGASA_API),
            pagasa_map: env_or("PAGASA_MAP_ROOT", DEFAULT_PAGASA_MAP),
            open_meteo: env_or("OPEN_METEO_ROOT", DEFAULT_OPEN_METEO),
            open_meteo_air: env_or("OPEN_METEO_AIR_ROOT", DEFAULT_OPEN_METEO_AIR),
            emb_aqi: std::env::var("EMB_AQI_URL").ok(),
            timeout: Duration::from_millis(
                std::env::var("PANAHON_HTTP_TIMEOUT_MS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(DEFAULT_TIMEOUT_MS),
            ),
        }
    }
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_owned())
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Meta {
    pub source: Vec<String>,
    pub quality: String,
    pub fetched_at: String,
    pub cached: bool,
    pub stale: bool,
    pub request_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Envelope<T: Serialize> {
    pub data: Option<T>,
    pub meta: Meta,
    pub errors: Vec<ApiError>,
}

#[derive(Clone, Debug)]
struct ProviderPayload {
    data: Value,
    source: Vec<String>,
    quality: String,
    errors: Vec<ApiError>,
    fetched_at: String,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    at: Instant,
    payload: ProviderPayload,
}

#[derive(Clone, Debug)]
struct CacheResult {
    payload: ProviderPayload,
    cached: bool,
    stale: bool,
}

#[derive(Default)]
struct Cache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    inflight: Mutex<HashSet<String>>,
    wake: Condvar,
}

impl Cache {
    fn get_or_fetch<F>(&self, key: &str, ttl: Duration, fetch: F) -> Result<CacheResult, ApiError>
    where
        F: FnOnce() -> Result<ProviderPayload, ApiError>,
    {
        loop {
            if let Some(entry) = self
                .entries
                .lock()
                .expect("cache poisoned")
                .get(key)
                .cloned()
            {
                if entry.at.elapsed() < ttl {
                    return Ok(CacheResult {
                        payload: entry.payload,
                        cached: true,
                        stale: false,
                    });
                }
            }

            let mut inflight = self.inflight.lock().expect("inflight poisoned");
            if inflight.insert(key.to_owned()) {
                break;
            }
            let _ = self
                .wake
                .wait_timeout(inflight, Duration::from_millis(250))
                .expect("wait failed");
        }

        let fetched = fetch();
        let result = match fetched {
            Ok(payload) => {
                self.entries.lock().expect("cache poisoned").insert(
                    key.to_owned(),
                    CacheEntry {
                        at: Instant::now(),
                        payload: payload.clone(),
                    },
                );
                Ok(CacheResult {
                    payload,
                    cached: false,
                    stale: false,
                })
            }
            Err(error) => {
                let stale = self
                    .entries
                    .lock()
                    .expect("cache poisoned")
                    .get(key)
                    .cloned();
                match stale {
                    Some(entry) => {
                        let mut payload = entry.payload;
                        payload.errors.push(error);
                        Ok(CacheResult {
                            payload,
                            cached: true,
                            stale: true,
                        })
                    }
                    None => Err(error),
                }
            }
        };

        self.inflight.lock().expect("inflight poisoned").remove(key);
        self.wake.notify_all();
        result
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    cache: Arc<Cache>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: Arc::new(Cache::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl HttpResponse {
    fn json<T: Serialize>(status: u16, value: &T) -> Self {
        let body = serde_json::to_vec(value)
            .unwrap_or_else(|_| b"{\"error\":\"serialization_failed\"}".to_vec());
        Self {
            status,
            content_type: "application/json; charset=utf-8".into(),
            body,
        }
    }

    fn error(status: u16, error: ApiError) -> Self {
        let envelope = Envelope::<Value> {
            data: None,
            meta: meta(Vec::new(), "unavailable", false, false),
            errors: vec![error],
        };
        Self::json(status, &envelope)
    }

    fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type: content_type.into(),
            body,
        }
    }
}

pub fn run(config: Config) -> std::io::Result<()> {
    let listener = TcpListener::bind(&config.bind)?;
    let state = Arc::new(AppState::new(config));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                thread::spawn(move || handle_connection(stream, state));
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) {
    let response = match read_request(&mut stream) {
        Ok(request) => handle_request(&state, &request),
        Err(error) => HttpResponse::error(400, error),
    };
    let _ = write_response(&mut stream, response);
}

pub fn handle_request(state: &AppState, request: &HttpRequest) -> HttpResponse {
    if request.method != "GET" {
        return HttpResponse::error(
            405,
            ApiError::new("method_not_allowed", "only GET is supported"),
        );
    }

    if request.path == "/health" {
        return health_response();
    }
    if request.path == "/api/v1/version" {
        return json_payload(
            json!({ "service": "panahon-weather-api", "api": "v1", "version": env!("CARGO_PKG_VERSION") }),
            vec!["local".into()],
            "official",
            Vec::new(),
            false,
            false,
        );
    }

    match request.path.as_str() {
        "/api/v1/locations/search" => locations(state, request),
        "/api/v1/weather" => weather(state, request),
        "/api/v1/air-quality" => air_quality(state, request),
        "/api/v1/alerts" => alerts(state, request),
        _ if request.path.starts_with("/api/v1/maps/") => maps(state, request),
        _ => HttpResponse::error(404, ApiError::new("not_found", "route not found")),
    }
}

fn health_response() -> HttpResponse {
    json_payload(
        json!({ "status": "ok", "service": "panahon-weather-api" }),
        vec!["local".into()],
        "official",
        Vec::new(),
        false,
        false,
    )
}

fn locations(state: &AppState, request: &HttpRequest) -> HttpResponse {
    let query = request.query.get("q").cloned().unwrap_or_default();
    if query.trim().chars().count() < 2 {
        return HttpResponse::error(
            400,
            ApiError::new("invalid_query", "q must contain at least two characters"),
        );
    }
    let key = format!("locations:{}", query.to_ascii_lowercase());
    let result = state
        .cache
        .get_or_fetch(&key, Duration::from_secs(3600), || {
            let local = local_locations(&query);
            if !local.is_empty() {
                return Ok(payload(
                    json!({ "query": query, "items": local }),
                    vec!["local".into()],
                    "official",
                ));
            }
            let url = format!(
                "{}/location/search?q={}",
                state.config.pagasa_api,
                encode_query(&query)
            );
            let value = get_json(&url, state.config.timeout)?;
            Ok(payload(
                normalize_locations(&query, &value),
                vec!["PAGASA".into()],
                "official",
            ))
        });
    response_from_cache(result)
}

fn weather(state: &AppState, request: &HttpRequest) -> HttpResponse {
    let Some((lat, lon)) = coordinates(request) else {
        return HttpResponse::error(
            400,
            ApiError::new(
                "invalid_coordinates",
                "lat and lon must be valid coordinates",
            ),
        );
    };
    let key = format!("weather:{lat:.4}:{lon:.4}");
    let result = state
        .cache
        .get_or_fetch(&key, Duration::from_secs(300), || {
            let pagasa = fetch_pagasa_weather(state, lat, lon);
            match pagasa {
                Ok(data) => Ok(data),
                Err(primary) => match fetch_open_meteo_weather(state, lat, lon) {
                    Ok(mut fallback) => {
                        fallback
                            .errors
                            .push(ApiError::new("pagasa_unavailable", primary.message));
                        fallback.quality = "forecast".into();
                        fallback.source.push("Open-Meteo".into());
                        Ok(fallback)
                    }
                    Err(_) => Err(primary),
                },
            }
        });
    response_from_cache(result)
}

fn air_quality(state: &AppState, request: &HttpRequest) -> HttpResponse {
    let Some((lat, lon)) = coordinates(request) else {
        return HttpResponse::error(
            400,
            ApiError::new(
                "invalid_coordinates",
                "lat and lon must be valid coordinates",
            ),
        );
    };
    let key = format!("air:{lat:.4}:{lon:.4}");
    let result = state
        .cache
        .get_or_fetch(&key, Duration::from_secs(300), || {
            if is_ncr(lat, lon) {
                let emb = match state.config.emb_aqi.as_deref() {
                    Some(url) => {
                        get_json(url, state.config.timeout).map(|value| normalize_emb_air(&value))
                    }
                    None => fetch_emb_powerbi(state),
                };
                if let Ok(value) = emb {
                    return Ok(payload(value, vec!["DENR-EMB".into()], "official"));
                }
            }
            fetch_cams(state, lat, lon)
        });
    response_from_cache(result)
}

fn alerts(state: &AppState, request: &HttpRequest) -> HttpResponse {
    let Some((lat, lon)) = coordinates(request) else {
        return HttpResponse::error(
            400,
            ApiError::new(
                "invalid_coordinates",
                "lat and lon must be valid coordinates",
            ),
        );
    };
    let key = format!("alerts:{lat:.4}:{lon:.4}");
    let result = state
        .cache
        .get_or_fetch(&key, Duration::from_secs(120), || {
            let url = format!(
                "{}/alerts?latitude={lat}&longitude={lon}",
                state.config.pagasa_api
            );
            let value = get_json(&url, state.config.timeout)?;
            Ok(payload(
                normalize_alerts(&value, lat, lon),
                vec!["PAGASA".into()],
                "official",
            ))
        });
    response_from_cache(result)
}

fn maps(state: &AppState, request: &HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = request.path.split('/').collect();
    if parts.len() < 5 {
        return HttpResponse::error(
            400,
            ApiError::new("invalid_map_route", "map route is incomplete"),
        );
    }
    let layer = parts[4];
    if parts.get(5) == Some(&"frames") {
        let index = request
            .query
            .get("index")
            .and_then(|value| value.parse::<usize>().ok())
            .or_else(|| parts.get(6).and_then(|value| value.parse().ok()))
            .unwrap_or(0);
        return map_frame(state, layer, index);
    }
    let key = format!("map:{layer}");
    let result = state.cache.get_or_fetch(&key, Duration::from_secs(60), || {
        let value = fetch_map_layer(state, layer)?;
        Ok(payload(value, vec!["PAGASA".into()], "official"))
    });
    response_from_cache(result)
}

fn map_frame(state: &AppState, layer: &str, index: usize) -> HttpResponse {
    if !matches!(layer, "radar" | "satellite") {
        return HttpResponse::error(
            400,
            ApiError::new(
                "unsupported_map_frame",
                "only radar and satellite expose image frames",
            ),
        );
    }
    match fetch_map_frame(state, layer, index) {
        Ok((content_type, bytes)) => HttpResponse::bytes(200, &content_type, bytes),
        Err(error) => HttpResponse::error(502, error),
    }
}

fn response_from_cache(result: Result<CacheResult, ApiError>) -> HttpResponse {
    match result {
        Ok(result) => {
            let meta = Meta {
                source: result.payload.source,
                quality: result.payload.quality,
                fetched_at: result.payload.fetched_at,
                cached: result.cached,
                stale: result.stale,
                request_id: request_id(),
            };
            HttpResponse::json(
                200,
                &Envelope {
                    data: Some(result.payload.data),
                    meta,
                    errors: result.payload.errors,
                },
            )
        }
        Err(error) => HttpResponse::error(502, error),
    }
}

fn json_payload(
    data: Value,
    source: Vec<String>,
    quality: &str,
    errors: Vec<ApiError>,
    cached: bool,
    stale: bool,
) -> HttpResponse {
    let envelope = Envelope {
        data: Some(data),
        meta: Meta {
            source,
            quality: quality.into(),
            fetched_at: now_iso(),
            cached,
            stale,
            request_id: request_id(),
        },
        errors,
    };
    HttpResponse::json(200, &envelope)
}

fn payload(data: Value, source: Vec<String>, quality: &str) -> ProviderPayload {
    ProviderPayload {
        data,
        source,
        quality: quality.into(),
        errors: Vec::new(),
        fetched_at: now_iso(),
    }
}

fn meta(source: Vec<String>, quality: &str, cached: bool, stale: bool) -> Meta {
    Meta {
        source,
        quality: quality.into(),
        fetched_at: now_iso(),
        cached,
        stale,
        request_id: request_id(),
    }
}

fn request_id() -> String {
    format!(
        "{}-{}",
        Utc::now().timestamp_millis(),
        REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn coordinates(request: &HttpRequest) -> Option<(f64, f64)> {
    let lat = request.query.get("lat")?.parse::<f64>().ok()?;
    let lon = request.query.get("lon")?.parse::<f64>().ok()?;
    (lat.is_finite()
        && lon.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lon))
    .then_some((lat, lon))
}

fn is_ncr(lat: f64, lon: f64) -> bool {
    (14.25..=14.95).contains(&lat) && (120.75..=121.25).contains(&lon)
}

fn fetch_pagasa_weather(state: &AppState, lat: f64, lon: f64) -> Result<ProviderPayload, ApiError> {
    let url = format!(
        "{}/nowcast/dashboard?latitude={lat}&longitude={lon}",
        state.config.pagasa_api
    );
    let value = get_json(&url, state.config.timeout)?;
    let observation_url = format!(
        "{}/observation/automatic?latitude={lat}&longitude={lon}",
        state.config.pagasa_api
    );
    let observation = get_json(&observation_url, state.config.timeout).ok();
    let mut normalized = normalize_pagasa_weather(&value, lat, lon);
    if let Some(observation) = observation.as_ref() {
        if let Some(current) = normalize_observation(observation) {
            normalized["current"] = current;
        }
    }
    Ok(payload(normalized, vec!["PAGASA".into()], "official"))
}

fn fetch_open_meteo_weather(
    state: &AppState,
    lat: f64,
    lon: f64,
) -> Result<ProviderPayload, ApiError> {
    let url = format!("{}?latitude={lat}&longitude={lon}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m,wind_direction_10m,visibility,uv_index&hourly=temperature_2m,precipitation,weather_code&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,weather_code,sunrise,sunset&timezone=auto", state.config.open_meteo);
    let value = get_json(&url, state.config.timeout)?;
    Ok(payload(
        normalize_open_meteo_weather(&value, lat, lon),
        vec!["Open-Meteo".into()],
        "forecast",
    ))
}

fn fetch_cams(state: &AppState, lat: f64, lon: f64) -> Result<ProviderPayload, ApiError> {
    let url = format!("{}?latitude={lat}&longitude={lon}&current=pm10,pm2_5,us_aqi&hourly=us_aqi&timezone=auto&past_days=1&forecast_days=1", state.config.open_meteo_air);
    let value = get_json(&url, state.config.timeout)?;
    Ok(payload(
        normalize_cams_air(&value),
        vec!["CAMS".into(), "Open-Meteo".into()],
        "forecast",
    ))
}

fn fetch_emb_powerbi(state: &AppState) -> Result<Value, ApiError> {
    let embed =
        BASE64.encode(json!({ "k": EMB_RESOURCE_KEY, "t": EMB_TENANT_ID, "m": 10 }).to_string());
    let view = get_text(
        &format!("{EMB_POWERBI_APP}/view?r={}", encode_query(&embed)),
        state.config.timeout,
    )?;
    let cluster = extract_cluster(&view)?;
    let model_url = format!(
        "{cluster}/public/reports/{EMB_RESOURCE_KEY}/modelsAndExploration?preferReadOnlySession=true"
    );
    let model = get_json_with_headers(
        &model_url,
        state.config.timeout,
        &[("X-PowerBI-ResourceKey", EMB_RESOURCE_KEY)],
    )?;
    let model_id = model
        .pointer("/exploration/report/modelId")
        .and_then(Value::as_u64)
        .ok_or_else(|| ApiError::new("emb_model_missing", "EMB Power BI modelId was missing"))?;
    let pm10 = run_emb_recipe(
        state,
        &cluster,
        model_id,
        include_str!("../../emb-aqi/recipes/cams-pm10.json"),
    )?;
    let pm25 = run_emb_recipe(
        state,
        &cluster,
        model_id,
        include_str!("../../emb-aqi/recipes/cams-pm25.json"),
    )?;
    let updated = run_emb_recipe(
        state,
        &cluster,
        model_id,
        include_str!("../../emb-aqi/recipes/cams-updated.json"),
    )?;

    let mut stations: HashMap<String, Map<String, Value>> = HashMap::new();
    for row in &pm10 {
        let Some(name) = map_text(row, &["CAMS.Station"]) else {
            continue;
        };
        let station = stations.entry(name.clone()).or_default();
        station.insert("name".into(), Value::String(name));
        station.insert(
            "address".into(),
            row.get("CAMS.Address").cloned().unwrap_or(Value::Null),
        );
        station.insert(
            "pm10".into(),
            map_number(row, &["Sum(CAMS.PM10AQI)"])
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
        station.insert(
            "pm10Remark".into(),
            row.get("CAMS.PM10REMARKS").cloned().unwrap_or(Value::Null),
        );
    }
    for row in &pm25 {
        let Some(name) = map_text(row, &["CAMS.Station"]) else {
            continue;
        };
        let station = stations.entry(name.clone()).or_default();
        station.entry("name").or_insert_with(|| Value::String(name));
        station
            .entry("address")
            .or_insert_with(|| row.get("CAMS.Address").cloned().unwrap_or(Value::Null));
        station.insert(
            "pm25".into(),
            map_number(row, &["Sum(CAMS.PM25AQI)"])
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
        station.insert(
            "pm25Remark".into(),
            row.get("CAMS.PM25REMARKS").cloned().unwrap_or(Value::Null),
        );
    }

    let mut output = Vec::new();
    for (_, station) in stations {
        let pm10 = station.get("pm10").and_then(Value::as_f64);
        let pm25 = station.get("pm25").and_then(Value::as_f64);
        let (aqi, pollutant) = match (pm10, pm25) {
            (Some(a), Some(b)) if a >= b => (Some(a), Some("PM10")),
            (Some(_), Some(b)) => (Some(b), Some("PM2.5")),
            (Some(a), None) => (Some(a), Some("PM10")),
            (None, Some(b)) => (Some(b), Some("PM2.5")),
            _ => (None, None),
        };
        let category = aqi
            .and_then(|value| u16::try_from(value.round() as i64).ok())
            .map(aqi_category);
        let mut station = Value::Object(station);
        if let Some(object) = station.as_object_mut() {
            let station_name = object
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("station")
                .to_owned();
            object.insert("id".into(), Value::String(slug(&station_name)));
            object.insert("aqi".into(), aqi.map(Value::from).unwrap_or(Value::Null));
            object.insert(
                "pollutant".into(),
                pollutant.map(Value::from).unwrap_or(Value::Null),
            );
            object.insert(
                "category".into(),
                category.map(Value::from).unwrap_or(Value::Null),
            );
        }
        output.push(station);
    }
    output.sort_by(|left, right| {
        right["aqi"]
            .as_f64()
            .partial_cmp(&left["aqi"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let values: Vec<f64> = output
        .iter()
        .filter_map(|station| station["aqi"].as_f64())
        .collect();
    let max = output.first();
    let updated_at = updated
        .iter()
        .find_map(|row| map_text(row, &["Min(CAMS.Date_Time)"]));
    Ok(json!({
        "provider": "DENR-EMB",
        "updatedAt": updated_at,
        "summary": {
            "stationCount": output.len(),
            "avgAqi": (!values.is_empty()).then(|| (values.iter().sum::<f64>() / values.len() as f64).round()),
            "maxAqi": max.and_then(|station| station["aqi"].as_f64()),
            "maxStation": max.and_then(|station| station["name"].as_str()),
            "category": max.and_then(|station| station["category"].as_str())
        },
        "stations": output,
        "trend": []
    }))
}

fn extract_cluster(html: &str) -> Result<String, ApiError> {
    let marker = "resolvedClusterUri = '";
    let (start, end_marker) = if let Some(start) = html.find(marker) {
        (start + marker.len(), "'")
    } else if let Some(start) = html.find("\"FixedClusterUri\":\"") {
        (start + "\"FixedClusterUri\":\"".len(), "\"")
    } else {
        return Err(ApiError::new(
            "emb_cluster_missing",
            "EMB Power BI cluster was missing",
        ));
    };
    let end = html[start..].find(end_marker).ok_or_else(|| {
        ApiError::new("emb_cluster_invalid", "EMB Power BI cluster was malformed")
    })? + start;
    let redirect = html[start..end].trim_end_matches('/');
    let api = redirect.replace("-redirect.", "-api.");
    if api == redirect {
        return Err(ApiError::new(
            "emb_cluster_invalid",
            "EMB Power BI cluster did not expose an API host",
        ));
    }
    Ok(api)
}

fn run_emb_recipe(
    state: &AppState,
    cluster: &str,
    model_id: u64,
    recipe: &str,
) -> Result<Vec<Map<String, Value>>, ApiError> {
    let mut body: Value = serde_json::from_str(recipe)
        .map_err(|error| ApiError::new("emb_recipe_invalid", error.to_string()))?;
    body["modelId"] = Value::from(model_id);
    let url = format!("{cluster}/public/reports/querydata?synchronous=true");
    let request_header = request_id();
    let activity_header = request_id();
    let response = post_json(
        &url,
        state.config.timeout,
        &[
            ("X-PowerBI-ResourceKey", EMB_RESOURCE_KEY),
            ("RequestId", request_header.as_str()),
            ("ActivityId", activity_header.as_str()),
        ],
        &body,
    )?;
    let data = response
        .pointer("/results/0/result/data")
        .ok_or_else(|| ApiError::new("emb_query_empty", "EMB Power BI query returned no data"))?;
    let dsr = data
        .get("dsr")
        .ok_or_else(|| ApiError::new("emb_query_empty", "EMB Power BI response had no DSR"))?;
    Ok(decode_dsr(dsr, data.get("descriptor")))
}

fn decode_dsr(dsr: &Value, descriptor: Option<&Value>) -> Vec<Map<String, Value>> {
    let mut rows = Vec::new();
    for dataset in dsr
        .get("DS")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let dicts = dataset
            .get("ValueDicts")
            .cloned()
            .unwrap_or_else(|| json!({}));
        for phase in dataset
            .get("PH")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for member in phase
                .as_object()
                .into_iter()
                .flat_map(|object| object.values())
            {
                let Some(items) = member.as_array() else {
                    continue;
                };
                let mut schema: Option<Vec<Value>> = None;
                for item in items {
                    if let Some(fields) = item.get("S").and_then(Value::as_array) {
                        schema = Some(fields.clone());
                    }
                    let Some(fields) = schema.as_ref() else {
                        continue;
                    };
                    let mut row = Map::new();
                    for (index, field) in fields.iter().enumerate() {
                        let name = field.get("N").and_then(Value::as_str).unwrap_or("unknown");
                        let raw = item
                            .get("C")
                            .and_then(Value::as_array)
                            .and_then(|values| values.get(index))
                            .or_else(|| item.get(name));
                        let value = raw
                            .and_then(|raw| decode_dsr_value(raw, field, &dicts))
                            .unwrap_or(Value::Null);
                        row.insert(name.into(), value);
                    }
                    rows.push(row);
                }
            }
        }
    }
    let mut aliases = HashMap::new();
    for select in descriptor
        .and_then(|value| value.get("Select"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(value), Some(name)) = (
            select.get("Value").and_then(Value::as_str),
            select.get("Name").and_then(Value::as_str),
        ) {
            aliases.insert(value.to_owned(), name.to_owned());
        }
    }
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(key, value)| (aliases.get(&key).cloned().unwrap_or(key), value))
                .collect()
        })
        .collect()
}

fn decode_dsr_value(raw: &Value, field: &Value, dicts: &Value) -> Option<Value> {
    if raw.is_null() {
        return None;
    }
    if let Some(dictionary) = field.get("DN").and_then(Value::as_str) {
        let index = raw.as_u64()? as usize;
        return dicts
            .get(dictionary)
            .and_then(Value::as_array)
            .and_then(|values| values.get(index))
            .cloned();
    }
    if let Some(text) = raw.as_str() {
        if let Some(number) = text
            .strip_suffix('L')
            .and_then(|value| value.parse::<i64>().ok())
        {
            return Some(Value::from(number));
        }
    }
    if field.get("T").and_then(Value::as_i64) == Some(7) {
        if let Some(milliseconds) = raw.as_i64() {
            return Some(Value::String(
                chrono::DateTime::<Utc>::from_timestamp_millis(milliseconds)?.to_rfc3339(),
            ));
        }
    }
    Some(raw.clone())
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
        } else if !result.ends_with('-') {
            result.push('-');
        }
    }
    result.trim_matches('-').to_owned()
}

fn fetch_map_layer(state: &AppState, layer: &str) -> Result<Value, ApiError> {
    let token = fetch_csrf(&state.config.pagasa_map, state.config.timeout)?;
    let (path, parameter) = match layer {
        "radar" => ("/api/v1/radar/timeline", Some("sublayer=mosaic-rainrate")),
        "satellite" => ("/api/v1/himawari/timeline", None),
        "lightning" => ("/api/v1/lightning", Some("parameter=ten_minute_frequency")),
        "typhoon" => ("/api/v1/cyclone-track", None),
        "hazards" => ("/api/v1/cap-alerts", None),
        _ => return Err(ApiError::new("unknown_map_layer", "unknown map layer")),
    };
    let suffix = parameter
        .map(|value| format!("?token={}&{}", encode_query(&token), value))
        .unwrap_or_else(|| format!("?token={}", encode_query(&token)));
    let value = get_json(
        &format!("{}{}{}", state.config.pagasa_map, path, suffix),
        state.config.timeout,
    )?;
    Ok(normalize_map(layer, &value))
}

fn fetch_map_frame(
    state: &AppState,
    layer: &str,
    index: usize,
) -> Result<(String, Vec<u8>), ApiError> {
    let token = fetch_csrf(&state.config.pagasa_map, state.config.timeout)?;
    let timeline = fetch_map_layer(state, layer)?;
    let frame = timeline
        .get("frames")
        .and_then(Value::as_array)
        .and_then(|items| items.get(index))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let url = if layer == "radar" {
        let time = text(&frame, &["observedAt", "observed_at", "t"]).unwrap_or_default();
        let version = text(&frame, &["tileVersion", "tile_version", "v"]).unwrap_or_default();
        format!("{}/api/v1/radar-data-image?token={}&sublayer=mosaic-rainrate&t={}&mode=webp&size=1024&v={}", state.config.pagasa_map, encode_query(&token), encode_query(&time), encode_query(&version))
    } else {
        let revision = text(&frame, &["revision"]).unwrap_or_default();
        let version = text(&frame, &["tileVersion", "tile_version"]).unwrap_or_default();
        format!("{}/api/v1/himawari-data?index={index}&size=1024&v={}-{}&token={}&parameter=himawari_ir_ext_smooth", state.config.pagasa_map, encode_query(&version), encode_query(&revision), encode_query(&token))
    };
    let (content_type, bytes) = get_bytes(&url, state.config.timeout)?;
    Ok((content_type.unwrap_or_else(|| "image/png".into()), bytes))
}

fn fetch_csrf(root: &str, timeout: Duration) -> Result<String, ApiError> {
    let html = get_text(root, timeout)?;
    let marker = "<meta name=\"csrf-token\" content=\"";
    let start = html.find(marker).ok_or_else(|| {
        ApiError::new(
            "map_token_missing",
            "PAGASA map page did not expose a CSRF token",
        )
    })? + marker.len();
    let end = html[start..]
        .find('\"')
        .ok_or_else(|| ApiError::new("map_token_invalid", "PAGASA map token was malformed"))?
        + start;
    Ok(html[start..end].to_owned())
}

fn get_json(url: &str, timeout: Duration) -> Result<Value, ApiError> {
    get_json_with_headers(url, timeout, &[])
}

fn get_json_with_headers(
    url: &str,
    timeout: Duration,
    headers: &[(&str, &str)],
) -> Result<Value, ApiError> {
    let mut request = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build()
        .get(url)
        .set("Accept", "application/json")
        .set("User-Agent", "panahon-weather-api/0.1");
    for (name, value) in headers {
        request = request.set(name, value);
    }
    let response = request
        .call()
        .map_err(|error| ApiError::new("upstream_unavailable", sanitize_error(error)))?;
    response
        .into_json()
        .map_err(|error| ApiError::new("invalid_upstream_json", sanitize_error(error)))
}

fn post_json(
    url: &str,
    timeout: Duration,
    headers: &[(&str, &str)],
    body: &Value,
) -> Result<Value, ApiError> {
    let mut request = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build()
        .post(url)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json;charset=UTF-8")
        .set("Origin", EMB_POWERBI_APP)
        .set("Referer", &format!("{EMB_POWERBI_APP}/"))
        .set("User-Agent", "panahon-weather-api/0.1");
    for (name, value) in headers {
        request = request.set(name, value);
    }
    let response = request
        .send_string(&body.to_string())
        .map_err(|error| ApiError::new("upstream_unavailable", sanitize_error(error)))?;
    response
        .into_json()
        .map_err(|error| ApiError::new("invalid_upstream_json", sanitize_error(error)))
}

fn get_text(url: &str, timeout: Duration) -> Result<String, ApiError> {
    let response = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build()
        .get(url)
        .set("Accept", "text/html")
        .set("User-Agent", "panahon-weather-api/0.1")
        .call()
        .map_err(|error| ApiError::new("upstream_unavailable", sanitize_error(error)))?;
    response
        .into_string()
        .map_err(|error| ApiError::new("invalid_upstream_text", sanitize_error(error)))
}

fn get_bytes(url: &str, timeout: Duration) -> Result<(Option<String>, Vec<u8>), ApiError> {
    let response = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build()
        .get(url)
        .set("Accept", "image/avif,image/webp,image/png,*/*")
        .set("User-Agent", "panahon-weather-api/0.1")
        .call()
        .map_err(|error| ApiError::new("upstream_unavailable", sanitize_error(error)))?;
    let content_type = response.header("content-type").map(str::to_owned);
    let mut body = Vec::new();
    response
        .into_reader()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut body)
        .map_err(|error| ApiError::new("image_read_failed", error.to_string()))?;
    if body.len() > 8 * 1024 * 1024 {
        return Err(ApiError::new(
            "image_too_large",
            "map image exceeded the 8 MiB limit",
        ));
    }
    Ok((content_type, body))
}

fn sanitize_error(error: impl std::fmt::Display) -> String {
    let message = error.to_string();
    if message.len() > 200 {
        message[..200].to_owned()
    } else {
        message
    }
}

fn normalize_pagasa_weather(value: &Value, lat: f64, lon: f64) -> Value {
    let current = object_from(
        value,
        &["/data/nc", "/data/nowcast", "/nowcast", "/current"],
    );
    let hourly = Value::Array(array_from(value, &["/data/hc", "/data/hourly", "/hourly"]));
    let daily = Value::Array(array_from(value, &["/data/dc", "/data/daily", "/daily"]));
    json!({ "location": { "latitude": lat, "longitude": lon }, "current": normalize_current(&current), "hourly": normalize_hourly(&hourly), "daily": normalize_daily(&daily), "supplementary": {} })
}

fn normalize_open_meteo_weather(value: &Value, lat: f64, lon: f64) -> Value {
    let current = value.get("current").cloned().unwrap_or_else(|| json!({}));
    let hourly = value.get("hourly").cloned().unwrap_or_else(|| json!({}));
    let daily = value.get("daily").cloned().unwrap_or_else(|| json!({}));
    json!({ "location": { "latitude": lat, "longitude": lon }, "current": normalize_current(&current), "hourly": normalize_open_meteo_hourly(&hourly), "daily": normalize_open_meteo_daily(&daily), "supplementary": { "sunrise": array_value(&daily, "sunrise", 0), "sunset": array_value(&daily, "sunset", 0), "uvIndex": number(&current, &["uv_index"]), "visibilityKm": number(&current, &["visibility"]).map(|v| v / 1000.0) } })
}

fn normalize_current(value: &Value) -> Value {
    json!({
        "temperatureC": number(value, &["tmp", "temperature_2m", "temperature", "temp"]),
        "feelsLikeC": number(value, &["hindex", "apparent_temperature", "feels_like", "heat_index"]),
        "humidityPercent": number(value, &["hum", "relative_humidity_2m", "humidity"]),
        "windKph": number(value, &["ws", "wind_speed_10m", "wind_kph", "wind_speed"]),
        "windDirection": text(value, &["wd", "wind_direction_10m", "wind_direction"]),
        "precipitationMm": number(value, &["precip_1h", "precipitation", "precipitation_sum"]),
        "condition": text(value, &["desc", "condition", "weather"]),
        "icon": text(value, &["icon", "weather_code"]),
        "updatedAt": text(value, &["time", "observed_at", "updated_at"])
    })
}

fn normalize_hourly(value: &Value) -> Vec<Value> {
    value.as_array().map(|items| items.iter().map(|item| json!({ "time": text(item, &["time"]), "temperatureC": number(item, &["tmp", "temperature", "temperature_2m"]), "precipitationMm": number(item, &["tprecip", "precipitation"]), "condition": text(item, &["desc", "condition"]), "icon": text(item, &["icon", "weather_code"]) })).collect()).unwrap_or_default()
}

fn normalize_daily(value: &Value) -> Vec<Value> {
    value.as_array().map(|items| items.iter().map(|item| json!({ "date": text(item, &["d", "date"]), "summary": text(item, &["desc", "summary"]), "lowC": number(item, &["tmin", "temperature_2m_min"]), "highC": number(item, &["tmax", "temperature_2m_max"]), "precipitationMm": number(item, &["tprecip", "precipitation_sum"]), "heatIndexC": number(item, &["hmax", "heat_index"]) })).collect()).unwrap_or_default()
}

fn normalize_open_meteo_hourly(value: &Value) -> Vec<Value> {
    let times = value
        .get("time")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    times.iter().enumerate().map(|(index, time)| json!({ "time": time, "temperatureC": array_value(value, "temperature_2m", index), "precipitationMm": array_value(value, "precipitation", index), "weatherCode": array_value(value, "weather_code", index) })).collect()
}

fn normalize_open_meteo_daily(value: &Value) -> Vec<Value> {
    let dates = value
        .get("time")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    dates.iter().enumerate().map(|(index, date)| json!({ "date": date, "lowC": array_value(value, "temperature_2m_min", index), "highC": array_value(value, "temperature_2m_max", index), "precipitationMm": array_value(value, "precipitation_sum", index), "weatherCode": array_value(value, "weather_code", index), "sunrise": array_value(value, "sunrise", index), "sunset": array_value(value, "sunset", index) })).collect()
}

fn normalize_emb_air(value: &Value) -> Value {
    let data = value.get("data").unwrap_or(value);
    json!({ "provider": "DENR-EMB", "updatedAt": data.get("updatedAt"), "dataClock": data.get("dataClock"), "summary": data.get("summary").cloned().unwrap_or_else(|| json!({})), "stations": data.get("stations").cloned().unwrap_or_else(|| json!([])), "trend": data.get("trend").cloned().unwrap_or_else(|| json!([])) })
}

fn normalize_cams_air(value: &Value) -> Value {
    let current = value.get("current").cloned().unwrap_or_else(|| json!({}));
    let pm10 = number(&current, &["pm10"]);
    let pm25 = number(&current, &["pm2_5"]);
    let aqi = number(&current, &["us_aqi"]);
    let station = json!({ "name": "Open-Meteo grid", "pm10": pm10, "pm25": pm25, "aqi": aqi, "category": aqi.and_then(|v| u16::try_from(v.round() as i64).ok()).map(aqi_category) });
    json!({ "provider": "CAMS", "updatedAt": current.get("time"), "summary": { "stationCount": 1, "avgAqi": aqi, "maxAqi": aqi, "maxStation": "Open-Meteo grid", "category": aqi.and_then(|v| u16::try_from(v.round() as i64).ok()).map(aqi_category) }, "stations": [station], "trend": value.get("hourly").and_then(|hourly| hourly.get("us_aqi")).cloned().unwrap_or_else(|| json!([])) })
}

fn normalize_alerts(value: &Value, lat: f64, lon: f64) -> Value {
    let items = array_from(value, &["/data", "/alerts", "/items"]);
    let alerts: Vec<Value> = items.iter().map(|item| json!({ "id": text(item, &["alert_id", "id"]), "kind": text(item, &["type", "kind"]), "title": text(item, &["title"]), "message": text(item, &["message", "detail"]), "area": area_text(item), "issuedAt": text(item, &["issued_at", "issued_at_at"]), "validUntil": text(item, &["valid_until_at", "valid_until"]) })).collect();
    json!({ "location": { "latitude": lat, "longitude": lon }, "items": alerts })
}

fn normalize_observation(value: &Value) -> Option<Value> {
    let item = array_from(value, &["/data", "/stations", "/items"])
        .into_iter()
        .next()?;
    Some(normalize_current(&item))
}

fn area_text(value: &Value) -> Option<String> {
    if let Some(text) = text(value, &["area", "location"]) {
        return Some(text);
    }
    value.get("locations").and_then(|locations| {
        locations
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|joined| !joined.is_empty())
            .or_else(|| locations.as_str().map(str::to_owned))
    })
}

fn normalize_locations(query: &str, value: &Value) -> Value {
    let items = array_from(value, &["/data", "/items", "/results"]);
    let locations: Vec<Value> = items.iter().enumerate().map(|(index, item)| json!({ "id": text(item, &["id"]).unwrap_or_else(|| format!("location-{index}")), "name": text(item, &["name", "municipality", "city"]), "region": text(item, &["province", "region"]).unwrap_or_else(|| "菲律宾".into()), "latitude": number(item, &["latitude", "lat"]), "longitude": number(item, &["longitude", "lon", "lng"]) })).collect();
    json!({ "query": query, "items": locations })
}

fn local_locations(query: &str) -> Vec<Value> {
    let lower = query.to_ascii_lowercase();
    let values = [
        ("manila", "马尼拉", "国家首都区", 14.5995, 120.9842),
        ("cebu", "宿务", "宿务省", 10.3157, 123.8854),
        ("davao", "达沃", "达沃地区", 7.1907, 125.4553),
    ];
    values.iter().filter(|(en, zh, _, _, _)| en.contains(&lower) || zh.contains(query)).map(|(en, zh, region, lat, lon)| json!({ "id": en, "name": zh, "nameEn": en, "region": region, "latitude": lat, "longitude": lon })).collect()
}

fn normalize_map(layer: &str, value: &Value) -> Value {
    let frames = array_from(
        value,
        &["/data/timeline", "/timeline", "/data/frames", "/frames"],
    );
    let normalized_frames: Vec<Value> = frames.iter().enumerate().map(|(index, item)| json!({ "index": index, "observedAt": text(item, &["observed_at", "observedAt", "time"]), "revision": text(item, &["revision"]), "tileVersion": text(item, &["tile_version", "tileVersion", "version"]) })).collect();
    json!({ "layer": layer, "frames": normalized_frames, "events": value.get("data").cloned().unwrap_or_else(|| json!([])) })
}

fn object_from(value: &Value, paths: &[&str]) -> Value {
    paths
        .iter()
        .find_map(|path| value.pointer(path).cloned())
        .filter(|value| value.is_object())
        .unwrap_or_else(|| json!({}))
}

fn array_from(value: &Value, paths: &[&str]) -> Vec<Value> {
    paths
        .iter()
        .find_map(|path| value.pointer(path).and_then(Value::as_array).cloned())
        .unwrap_or_default()
}

fn array_value(value: &Value, field: &str, index: usize) -> Option<Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .and_then(|items| items.get(index))
        .cloned()
}

fn number(value: &Value, fields: &[&str]) -> Option<f64> {
    fields.iter().find_map(|field| {
        value
            .get(*field)
            .and_then(|item| item.as_f64().or_else(|| item.as_str()?.trim().parse().ok()))
    })
}

fn map_number(value: &Map<String, Value>, fields: &[&str]) -> Option<f64> {
    fields.iter().find_map(|field| {
        value
            .get(*field)
            .and_then(|item| item.as_f64().or_else(|| item.as_str()?.trim().parse().ok()))
    })
}

fn text(value: &Value, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        value.get(*field).and_then(|item| {
            item.as_str()
                .map(str::to_owned)
                .or_else(|| item.as_i64().map(|number| number.to_string()))
        })
    })
}

fn map_text(value: &Map<String, Value>, fields: &[&str]) -> Option<String> {
    fields.iter().find_map(|field| {
        value.get(*field).and_then(|item| {
            item.as_str()
                .map(str::to_owned)
                .or_else(|| item.as_i64().map(|number| number.to_string()))
        })
    })
}

fn aqi_category(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "优",
        51..=100 => "良",
        101..=150 => "轻度污染",
        151..=200 => "中度污染",
        201..=300 => "重度污染",
        _ => "严重污染",
    }
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, ApiError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| ApiError::new("request_read_failed", error.to_string()))?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") && bytes.len() < 64 * 1024 {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| ApiError::new("request_read_failed", error.to_string()))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| ApiError::new("invalid_request", "request was not valid UTF-8"))?;
    let line = text
        .lines()
        .next()
        .ok_or_else(|| ApiError::new("invalid_request", "request line is missing"))?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default();
    let (path, query) = split_target(target);
    Ok(HttpRequest {
        method,
        path,
        query,
    })
}

fn split_target(target: &str) -> (String, HashMap<String, String>) {
    let mut pieces = target.splitn(2, '?');
    let path = pieces.next().unwrap_or("/").to_owned();
    let query = pieces
        .next()
        .unwrap_or_default()
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut pair = part.splitn(2, '=');
            let key = percent_decode(pair.next().unwrap_or_default());
            let value = percent_decode(pair.next().unwrap_or_default());
            (key, value)
        })
        .collect();
    (path, query)
}

fn write_response(stream: &mut TcpStream, response: HttpResponse) -> std::io::Result<()> {
    let reason = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        502 => "Bad Gateway",
        _ => "Internal Server Error",
    };
    write!(stream, "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", response.status, reason, response.content_type, response.body.len())?;
    stream.write_all(&response.body)
}

fn encode_query(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                output.push(byte);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn request(path: &str) -> HttpRequest {
        let (path, query) = split_target(path);
        HttpRequest {
            method: "GET".into(),
            path,
            query,
        }
    }

    #[test]
    fn validates_coordinates_and_query_length() {
        let state = AppState::new(Config::from_env());
        let response = handle_request(&state, &request("/api/v1/weather?lat=91&lon=0"));
        assert_eq!(response.status, 400);
        let response = handle_request(&state, &request("/api/v1/locations/search?q=a"));
        assert_eq!(response.status, 400);
    }

    #[test]
    fn parses_percent_encoded_query() {
        let (path, query) = split_target("/api/v1/locations/search?q=New%20York");
        assert_eq!(path, "/api/v1/locations/search");
        assert_eq!(query.get("q"), Some(&"New York".to_string()));
    }

    #[test]
    fn local_location_search_is_available_without_network() {
        let state = AppState::new(Config::from_env());
        let response = handle_request(&state, &request("/api/v1/locations/search?q=man"));
        assert_eq!(response.status, 200);
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["data"]["items"][0]["name"], "马尼拉");
    }

    #[test]
    fn categories_match_denr_bands() {
        assert_eq!(aqi_category(50), "优");
        assert_eq!(aqi_category(51), "良");
        assert_eq!(aqi_category(301), "严重污染");
    }

    #[test]
    fn cache_returns_stale_payload_after_failure() {
        let cache = Cache::default();
        let first = cache
            .get_or_fetch("key", Duration::from_secs(0), || {
                Ok(payload(
                    json!({ "ok": true }),
                    vec!["test".into()],
                    "official",
                ))
            })
            .unwrap();
        assert!(!first.stale);
        let second = cache
            .get_or_fetch("key", Duration::from_secs(0), || {
                Err(ApiError::new("failed", "upstream failed"))
            })
            .unwrap();
        assert!(second.stale);
        assert_eq!(second.payload.data["ok"], true);
    }

    #[test]
    fn decodes_powerbi_dictionary_rows() {
        let dsr = json!({
            "DS": [{
                "ValueDicts": {"D0": ["Navotas"]},
                "PH": [{"DM0": [
                    {"S": [{"N": "G0", "DN": "D0"}, {"N": "G1"}], "C": [0, "44L"]}
                ]}]
            }]
        });
        let descriptor = json!({"Select": [
            {"Value": "G0", "Name": "CAMS.Station"},
            {"Value": "G1", "Name": "Sum(CAMS.PM10AQI)"}
        ]});
        let rows = decode_dsr(&dsr, Some(&descriptor));
        assert_eq!(rows[0]["CAMS.Station"], "Navotas");
        assert_eq!(rows[0]["Sum(CAMS.PM10AQI)"], 44);
    }
}

