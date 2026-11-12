//! Application state and the five screens.
//!
//! Every network call runs on a worker thread and reports back through a
//! channel, so the UI keeps painting while a request is in flight.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Align2, Color32, FontId, Layout, Pos2, RichText, Sense, Vec2};
use panahon_core::{
    fetch_air_quality_at, fetch_api_version, fetch_public_dashboard, fetch_remote_map_layer,
    fetch_remote_map_layer_frame, search_places, AirQuality, ApiVersion, Dashboard, DataQuality,
    DataSource, MapTrackPoint, Place, RemoteMapSnapshot,
};

#[cfg(target_os = "android")]
use panahon_core::OFFICIAL_MAP_URL;

use crate::theme::{self, Palette};
use crate::widgets;

struct MapLayer {
    key: &'static str,
    title_zh: &'static str,
    title_en: &'static str,
    supported: bool,
}

const MAP_LAYERS: [MapLayer; 7] = [
    MapLayer {
        key: "radar",
        title_zh: "降水雷达",
        title_en: "Precipitation radar",
        supported: true,
    },
    MapLayer {
        key: "satellite",
        title_zh: "卫星",
        title_en: "Satellite",
        supported: true,
    },
    MapLayer {
        key: "lightning",
        title_zh: "闪电",
        title_en: "Lightning",
        supported: true,
    },
    MapLayer {
        key: "typhoon",
        title_zh: "台风",
        title_en: "Typhoon",
        supported: true,
    },
    MapLayer {
        key: "hazard",
        title_zh: "灾害",
        title_en: "Hazard",
        supported: true,
    },
    MapLayer {
        key: "temperature",
        title_zh: "温度",
        title_en: "Temperature",
        supported: false,
    },
    MapLayer {
        key: "wind",
        title_zh: "风力",
        title_en: "Wind",
        supported: false,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiLanguage {
    Chinese,
    English,
}

impl UiLanguage {
    fn from_runtime() -> Self {
        let value = std::env::var("PANAHON_LANG")
            .ok()
            .or_else(|| crate::read_preference("language"))
            .unwrap_or_default();
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" => Self::English,
            _ => Self::Chinese,
        }
    }

    fn is_english(self) -> bool {
        matches!(self, Self::English)
    }
}

fn encode_place(place: &Place) -> String {
    format!(
        "{}|{}|{}|{:.6}|{:.6}",
        place.id, place.name, place.region, place.latitude, place.longitude
    )
}

fn decode_place(value: &str) -> Option<Place> {
    let mut parts = value.splitn(5, '|');
    Some(Place {
        id: parts.next()?.to_owned(),
        name: parts.next()?.to_owned(),
        region: parts.next()?.to_owned(),
        latitude: parts.next()?.parse().ok()?,
        longitude: parts.next()?.parse().ok()?,
    })
}

fn localized_error(error: &str, is_english: bool) -> String {
    if !is_english {
        return error.to_owned();
    }
    match error {
        "网络适配器未启用" => "Network adapter is disabled".into(),
        "请输入至少 2 个字符" => "Enter at least 2 characters".into(),
        "地点响应格式无法解析" => "Location response could not be parsed".into(),
        _ => error.to_owned(),
    }
}

fn setting_text(key: &str, english: bool, android: bool) -> &'static str {
    match (key, english, android) {
        ("runtime_status", true, _) => "Runtime status",
        ("runtime_status", false, _) => "运行状态",
        ("weather_cache", true, _) => "Weather cache",
        ("weather_cache", false, _) => "天气缓存",
        ("available", true, _) => "available",
        ("available", false, _) => "可用",
        ("unavailable", true, _) => "not available",
        ("unavailable", false, _) => "不可用",
        ("notifications", true, _) => "Notifications",
        ("notifications", false, _) => "通知",
        ("permissions", true, _) => "Permissions",
        ("permissions", false, _) => "权限",
        ("permission_android", true, true) => "location selected manually; permission is optional",
        ("permission_android", false, true) => "当前地点由用户选择；定位权限为可选",
        ("permission_android", true, false) => {
            "desktop uses the selected city; no location permission"
        }
        ("permission_android", false, false) => "桌面使用已选城市；不请求定位权限",
        _ => "—",
    }
}

enum Msg {
    Dashboard(u64, Result<Dashboard, String>),
    Air(u64, Result<AirQuality, String>),
    Version(Result<ApiVersion, String>),
    Places(Result<Vec<Place>, String>),
    Map(
        u64,
        Box<(String, Option<usize>, Result<RemoteMapSnapshot, String>)>,
    ),
}

pub struct PanahonApp {
    tab: usize,
    place_generation: u64,
    map_generation: u64,
    cached: bool,
    language: UiLanguage,
    dashboard: Dashboard,
    status: String,
    loading_weather: bool,
    air: Option<AirQuality>,
    air_history: Vec<(String, u16)>,
    air_status: Option<String>,
    loading_air: bool,
    map_layer: usize,
    map_status: String,
    map_source: String,
    map_quality: DataQuality,
    map_updated_at: Option<String>,
    map_detail: Option<String>,
    map_track: Vec<MapTrackPoint>,
    map_texture: Option<egui::TextureHandle>,
    map_size: Option<[usize; 2]>,
    map_frame_count: Option<u32>,
    map_frame_index: u32,
    map_playing: bool,
    map_next_frame_at: Option<Instant>,
    loading_map: bool,
    search_query: String,
    search_results: Vec<Place>,
    searching: bool,
    search_focus: bool,
    search_attempted: bool,
    version: Option<String>,
    notification_status: String,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    /// Android 状态栏 / 手势条高度（点），读取失败时为 None
    insets_points: Option<(f32, f32)>,
    inset_attempts: u32,
}

impl PanahonApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cjk_font(&cc.egui_ctx);
        theme::apply_style(&cc.egui_ctx);

        let (tx, rx) = channel();
        // 桌面预览时可以用 PANAHON_TAB=1..4 直接打开某个页面截图
        let initial_tab = std::env::var("PANAHON_TAB")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value < 5)
            .unwrap_or(0);
        let language = UiLanguage::from_runtime();
        let place = crate::read_preference("place")
            .as_deref()
            .and_then(decode_place)
            .unwrap_or_else(|| Dashboard::sample().place.clone());
        let notifications_enabled = crate::read_preference("notifications_enabled")
            .as_deref()
            .is_some_and(|value| value == "1");
        let mut app = Self {
            tab: initial_tab,
            place_generation: 0,
            map_generation: 0,
            cached: true,
            language,
            dashboard: if std::env::var("PANAHON_DEMO").as_deref() == Ok("1") {
                Dashboard::sample()
            } else {
                crate::read_preference("weather_cache_v2")
                    .and_then(|raw| serde_json::from_str::<Dashboard>(&raw).ok())
                    .filter(|data| data.place == place && data.source == DataSource::PublicApi)
                    .unwrap_or_else(|| Dashboard::empty(place))
            },
            status: if language.is_english() {
                "Loading official data…".into()
            } else {
                "正在读取官方数据…".into()
            },
            loading_weather: false,
            air: None,
            air_history: load_air_history(),
            air_status: None,
            loading_air: false,
            map_layer: 0,
            map_status: if language.is_english() {
                "Choose a layer, then refresh".into()
            } else {
                "选择图层后点刷新".into()
            },
            map_source: "PAGASA".into(),
            map_quality: DataQuality::Unavailable,
            map_updated_at: None,
            map_detail: None,
            map_track: Vec::new(),
            map_texture: None,
            map_size: None,
            map_frame_count: None,
            map_frame_index: 0,
            map_playing: false,
            map_next_frame_at: None,
            loading_map: false,
            search_query: String::new(),
            search_results: Vec::new(),
            searching: false,
            search_focus: false,
            search_attempted: false,
            version: None,
            notification_status: if notifications_enabled {
                "每 30 分钟提醒一次".into()
            } else {
                "未启用".into()
            },
            tx,
            rx,
            insets_points: None,
            inset_attempts: 0,
        };
        #[cfg(target_os = "android")]
        if notifications_enabled {
            let _ = crate::start_local_notification_scheduler();
        }
        if std::env::var("PANAHON_DEMO").as_deref() != Ok("1") {
            app.refresh_weather(&cc.egui_ctx);
            app.refresh_air(&cc.egui_ctx);
        }
        app
    }

    fn refresh_weather(&mut self, ctx: &egui::Context) {
        if self.loading_weather {
            return;
        }
        self.loading_weather = true;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let place = self.dashboard.place.clone();
        let generation = self.place_generation;
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Dashboard(generation, fetch_public_dashboard(&place)));
            ctx.request_repaint();
            let version = fetch_api_version();
            let _ = tx.send(Msg::Version(version));
            ctx.request_repaint();
        });
    }

    fn refresh_air(&mut self, ctx: &egui::Context) {
        if self.loading_air {
            return;
        }
        self.loading_air = true;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let latitude = self.dashboard.place.latitude;
        let longitude = self.dashboard.place.longitude;
        let generation = self.place_generation;
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Air(
                generation,
                fetch_air_quality_at(latitude, longitude),
            ));
            ctx.request_repaint();
        });
    }

    fn refresh_map(&mut self, ctx: &egui::Context) {
        if self.loading_map {
            return;
        }
        self.loading_map = true;
        self.map_generation += 1;
        let generation = self.map_generation;
        self.map_status = if self.language.is_english() {
            "Reading official layer…".into()
        } else {
            "正在读取官方图层…".into()
        };
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let layer = &MAP_LAYERS[self.map_layer];
        let key = layer.key.to_owned();
        let title = if self.language.is_english() {
            layer.title_en
        } else {
            layer.title_zh
        }
        .to_owned();
        if !layer.supported {
            self.loading_map = false;
            self.map_quality = DataQuality::Unavailable;
            self.map_updated_at = None;
            self.map_status = if self.language.is_english() {
                format!("{title} layer has no data adapter")
            } else {
                format!("{title}图层暂无数据适配器")
            };
            return;
        }
        let frame_index = self.map_frame_count.map(|_| self.map_frame_index as usize);
        std::thread::spawn(move || {
            let result = match frame_index {
                Some(index) if matches!(key.as_str(), "radar" | "satellite") => {
                    fetch_remote_map_layer_frame(&key, index)
                }
                _ => fetch_remote_map_layer(&key),
            };
            let _ = tx.send(Msg::Map(generation, Box::new((title, frame_index, result))));
            ctx.request_repaint();
        });
    }

    fn tick_map_playback(&mut self, ctx: &egui::Context) {
        if !self.map_playing {
            return;
        }
        let Some(count) = self.map_frame_count.filter(|count| *count > 1) else {
            self.map_playing = false;
            self.map_next_frame_at = None;
            return;
        };
        let now = Instant::now();
        let due = self
            .map_next_frame_at
            .map(|deadline| now >= deadline)
            .unwrap_or(true);
        if due {
            self.map_frame_index = (self.map_frame_index + 1) % count;
            self.map_next_frame_at = Some(now + Duration::from_secs(4));
            if !self.loading_map {
                self.refresh_map(ctx);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    fn search(&mut self, ctx: &egui::Context) {
        if self.searching || self.search_query.trim().chars().count() < 2 {
            return;
        }
        self.searching = true;
        self.search_attempted = true;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let query = self.search_query.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Places(search_places(&query)));
            ctx.request_repaint();
        });
    }

    fn drain(&mut self, ctx: &egui::Context) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                Msg::Dashboard(generation, _) | Msg::Air(generation, _)
                    if generation != self.place_generation => {}
                Msg::Dashboard(_, Ok(dashboard)) => {
                    if let Ok(raw) = serde_json::to_string(&dashboard) {
                        crate::write_preference("weather_cache_v2", &raw);
                    }
                    self.cached = false;
                    self.dashboard = dashboard;
                    self.status = if self.language.is_english() {
                        "Updated".into()
                    } else {
                        "已更新".into()
                    };
                    self.loading_weather = false;
                }
                Msg::Dashboard(_, Err(error)) => {
                    self.cached = true;
                    self.status = if self.language.is_english() {
                        format!("Refresh failed: {}", localized_error(&error, true))
                    } else {
                        format!("刷新失败：{error}")
                    };
                    self.loading_weather = false;
                }
                Msg::Air(_, Ok(air)) => {
                    self.record_air_history(&air);
                    self.air = Some(air);
                    self.air_status = None;
                    self.loading_air = false;
                }
                Msg::Air(_, Err(error)) => {
                    self.air_status = Some(localized_error(&error, self.language.is_english()));
                    self.loading_air = false;
                }
                Msg::Version(Ok(version)) => self.version = Some(version.latest_version),
                Msg::Version(Err(_)) => {}
                Msg::Places(Ok(places)) => {
                    self.search_results = places;
                    self.searching = false;
                }
                Msg::Places(Err(error)) => {
                    self.status = if self.language.is_english() {
                        format!("Location search failed: {}", localized_error(&error, true))
                    } else {
                        format!("地点搜索失败：{error}")
                    };
                    self.searching = false;
                }
                Msg::Map(generation, _) if generation != self.map_generation => {}
                Msg::Map(_, payload) => {
                    let (title, requested_frame, result) = *payload;
                    self.loading_map = false;
                    if requested_frame.is_some_and(|index| index != self.map_frame_index as usize) {
                        self.refresh_map(ctx);
                        continue;
                    }
                    match result {
                        Ok(snapshot) => {
                            self.map_source = snapshot.source.clone();
                            self.map_quality = snapshot.quality;
                            self.map_updated_at = snapshot.latest_at.clone();
                            self.map_detail = snapshot.summary.clone();
                            self.map_track = snapshot.track.clone();
                            self.map_frame_count = snapshot.frame_count;
                            if let Some(count) = snapshot.frame_count {
                                self.map_frame_index = requested_frame
                                    .map(|index| index as u32)
                                    .unwrap_or_else(|| count.saturating_sub(1))
                                    .min(count.saturating_sub(1));
                            } else {
                                self.map_frame_index = 0;
                            }
                            let latest = snapshot.latest_at.clone().unwrap_or_else(|| {
                                if self.language.is_english() {
                                    "unknown".into()
                                } else {
                                    "未知".into()
                                }
                            });
                            let count = snapshot
                                .frame_count
                                .map(|value| {
                                    if self.language.is_english() {
                                        format!("{value} frames")
                                    } else {
                                        format!("{value} 帧")
                                    }
                                })
                                .or_else(|| {
                                    snapshot.item_count.map(|value| {
                                        if self.language.is_english() {
                                            format!("{value} items")
                                        } else {
                                            format!("{value} 条")
                                        }
                                    })
                                })
                                .unwrap_or_else(|| {
                                    if self.language.is_english() {
                                        "returned".into()
                                    } else {
                                        "已返回".into()
                                    }
                                });
                            self.map_status = if self.language.is_english() {
                                format!("{title}: {count}, latest {latest}")
                            } else {
                                format!("{title}：{count}，最新 {latest}")
                            };
                            self.map_texture = snapshot.image.and_then(|bytes| {
                                image::load_from_memory(&bytes).ok().map(|decoded| {
                                    let rgba = decoded.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        rgba.as_flat_samples().as_slice(),
                                    );
                                    self.map_size = Some(size);
                                    ctx.load_texture(
                                        "map-layer",
                                        pixels,
                                        egui::TextureOptions::LINEAR,
                                    )
                                })
                            });
                        }
                        Err(error) => {
                            self.map_quality = DataQuality::Unavailable;
                            self.map_updated_at = None;
                            self.map_detail = None;
                            self.map_track.clear();
                            self.map_texture = None;
                            self.map_size = None;
                            self.map_status = if self.language.is_english() {
                                format!("{title} failed: {}", localized_error(&error, true))
                            } else {
                                format!("{title}读取失败：{error}")
                            };
                        }
                    }
                }
            }
        }
    }

    fn record_air_history(&mut self, air: &AirQuality) {
        let Some(aqi) = air.summary.avg_aqi else {
            return;
        };
        let at = air
            .data_clock
            .clone()
            .or_else(|| air.updated_at.clone())
            .unwrap_or_else(|| {
                if self.language.is_english() {
                    "current".into()
                } else {
                    "当前".into()
                }
            });
        if self
            .air_history
            .last()
            .map(|(last_at, _)| last_at == &at)
            .unwrap_or(false)
        {
            return;
        }
        self.air_history.push((at, aqi));
        if self.air_history.len() > 48 {
            let excess = self.air_history.len() - 48;
            self.air_history.drain(0..excess);
        }
        let encoded = self
            .air_history
            .iter()
            .map(|(at, aqi)| format!("{at}|{aqi}"))
            .collect::<Vec<_>>()
            .join(";");
        crate::write_preference("air_history", &encoded);
    }

    fn palette(&self) -> Palette {
        theme::palette(self.dashboard.current.icon, self.dashboard.current.is_day)
    }

    /// 状态栏和手势条高度来自 Android 的 WindowInsets，只读取一次。
    fn refresh_insets(&mut self, ctx: &egui::Context) {
        if self.insets_points.is_some() || self.inset_attempts > 240 {
            return;
        }
        self.inset_attempts += 1;

        #[cfg(target_os = "android")]
        {
            let ppp = ctx.pixels_per_point().max(0.1);
            if let Some((top, bottom)) = crate::window_insets_px() {
                if top > 0.0 || bottom > 0.0 {
                    self.insets_points = Some((top / ppp, bottom / ppp));
                }
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = ctx;
            self.insets_points = Some((theme::safe_top(), 0.0));
        }
    }

    // ---------------------------------------------------------------- weather

    fn weather_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        widgets::hero(ui, &self.dashboard, self.air.as_ref(), palette);
        ui.add_space(14.0);

        widgets::blender_sky_card(ui, palette);
        ui.add_space(10.0);

        widgets::hourly_strip(ui, &self.dashboard.hourly, palette);
        ui.add_space(10.0);

        widgets::daily_list(ui, &self.dashboard.forecast, palette);
        ui.add_space(10.0);

        match (&self.air, &self.air_status) {
            (Some(air), _) => {
                let response = ui
                    .scope(|ui| widgets::aqi_summary(ui, air, palette))
                    .response;
                if response.interact(Sense::click()).clicked() {
                    self.tab = 1;
                }
            }
            (None, Some(error)) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    theme::section_title(
                        ui,
                        if self.language.is_english() {
                            "Air quality"
                        } else {
                            "空气质量"
                        },
                        palette,
                    );
                    ui.label(
                        RichText::new(widgets::shorten(error, 90))
                            .size(11.0)
                            .color(palette.dim),
                    );
                });
            }
            (None, None) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    theme::section_title(
                        ui,
                        if self.language.is_english() {
                            "Air quality"
                        } else {
                            "空气质量"
                        },
                        palette,
                    );
                    widgets::loading_row(
                        ui,
                        palette,
                        if self.language.is_english() {
                            "Reading DENR-EMB stations…"
                        } else {
                            "正在读取 DENR-EMB 站点数据…"
                        },
                    );
                });
            }
        }
        ui.add_space(10.0);

        self.map_entry(ui, palette);
        ui.add_space(10.0);

        widgets::detail_grid(ui, &self.dashboard.current, palette);
        ui.add_space(10.0);

        widgets::sun_details(ui, &self.dashboard.sun, palette);
        ui.add_space(10.0);

        widgets::life_indices(ui, &self.dashboard.life_indices(), palette);
        ui.add_space(10.0);

        if let Some(alert) = self.dashboard.alerts.first() {
            let response = ui
                .scope(|ui| widgets::alert_card(ui, alert, palette, self.language.is_english()))
                .response;
            if response.interact(Sense::click()).clicked() {
                self.tab = 3;
            }
            if self.dashboard.alerts.len() > 1 {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(if self.language.is_english() {
                        format!(
                            "{} more alerts · open Alerts",
                            self.dashboard.alerts.len() - 1
                        )
                    } else {
                        format!(
                            "另有 {} 条预警，见「预警」页",
                            self.dashboard.alerts.len() - 1
                        )
                    })
                    .size(11.0)
                    .color(palette.faint),
                );
            }
            ui.add_space(10.0);
        }

        if let Some(url) = &self.dashboard.outlook_url {
            theme::card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                theme::section_title(
                    ui,
                    if self.language.is_english() {
                        "City outlook"
                    } else {
                        "城市展望"
                    },
                    palette,
                );
                ui.label(
                    RichText::new(if self.language.is_english() {
                        "PAGASA public video forecast"
                    } else {
                        "PAGASA 公开城市展望视频"
                    })
                    .size(11.0)
                    .color(palette.faint),
                );
                ui.add_space(5.0);
                #[cfg(target_os = "android")]
                if widgets::pill_button(
                    ui,
                    if self.language.is_english() {
                        "Open outlook"
                    } else {
                        "打开城市展望"
                    },
                    palette,
                )
                .clicked()
                {
                    crate::open_official_map(url);
                }
                #[cfg(not(target_os = "android"))]
                ui.label(RichText::new(url).size(10.0).color(palette.faint));
            });
            ui.add_space(10.0);
        }

        self.footer(ui, palette);
    }

    // ------------------------------------------------------------ air quality

    fn air_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::pill_button(
                    ui,
                    if self.language.is_english() {
                        "Refresh"
                    } else {
                        "刷新"
                    },
                    palette,
                )
                .clicked()
                {
                    let ctx = ui.ctx().clone();
                    self.refresh_air(&ctx);
                }
            });
        });
        ui.add_space(10.0);

        match (&self.air, &self.air_status) {
            (Some(air), _) => {
                widgets::aqi_summary(ui, air, palette);
                ui.add_space(10.0);
                widgets::aqi_pollutants(ui, air, palette);
                ui.add_space(10.0);

                widgets::aqi_trend(ui, &air.trend, palette);
                ui.add_space(10.0);
                widgets::aqi_history(ui, &self.air_history, palette);
                ui.add_space(10.0);

                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    theme::section_title(
                        ui,
                        if self.language.is_english() {
                            "Stations (highest AQI first)"
                        } else {
                            "监测站（AQI 从高到低）"
                        },
                        palette,
                    );
                    ui.add_space(6.0);
                    for (index, station) in air.stations.iter().enumerate() {
                        if index > 0 {
                            let (line, _) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 1.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                line,
                                egui::CornerRadius::ZERO,
                                palette.card_line,
                            );
                            ui.add_space(6.0);
                        }
                        let color = widgets::aqi_color(station.aqi);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(if self.language.is_english() {
                                        station.name.as_deref().unwrap_or(&station.name_zh)
                                    } else {
                                        &station.name_zh
                                    })
                                    .size(13.0)
                                    .color(palette.text),
                                );
                                let mut detail = String::new();
                                if let Some(pm25) = station.pm25 {
                                    detail.push_str(&format!("PM2.5 {pm25:.0}"));
                                }
                                if let Some(pm10) = station.pm10 {
                                    if !detail.is_empty() {
                                        detail.push_str(" · ");
                                    }
                                    detail.push_str(&format!("PM10 {pm10:.0}"));
                                }
                                if let Some(pollutant) = &station.pollutant {
                                    if !detail.is_empty() {
                                        detail.push_str(" · ");
                                    }
                                    detail.push_str(&if self.language.is_english() {
                                        format!("Primary {pollutant}")
                                    } else {
                                        format!("首要 {pollutant}")
                                    });
                                }
                                ui.label(RichText::new(detail).size(10.0).color(palette.faint));
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(widgets::aqi_category_text(
                                        &station.category,
                                        self.language.is_english(),
                                    ))
                                    .size(11.0)
                                    .color(color),
                                );
                                widgets::aqi_badge(ui, station.aqi);
                            });
                        });
                    }
                });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(&air.source.note)
                        .size(10.0)
                        .color(palette.faint),
                );
            }
            (None, Some(error)) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(if self.language.is_english() {
                            "Official air quality data is unavailable"
                        } else {
                            "暂时读不到官方空气质量数据"
                        })
                        .size(15.0)
                        .strong()
                        .color(palette.text),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(localized_error(error, self.language.is_english()))
                            .size(11.0)
                            .color(palette.dim),
                    );
                });
            }
            (None, None) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    widgets::loading_row(
                        ui,
                        palette,
                        if self.language.is_english() {
                            "Reading DENR-EMB report…"
                        } else {
                            "正在读取 DENR-EMB 报表…"
                        },
                    );
                });
            }
        }
    }

    // -------------------------------------------------------------------- map

    /// 地图页：华为把地图放进一张圆角卡片，顶部是图层分页，下面是图像视口。
    fn map_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let is_english = self.language.is_english();

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            widgets::card_header_public(
                ui,
                palette,
                if is_english {
                    "Weather map"
                } else {
                    "天气地图"
                },
                &self.map_source,
            );
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                for (index, layer) in MAP_LAYERS.iter().enumerate() {
                    let active = index == self.map_layer;
                    let fill = if active {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 210)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 26)
                    };
                    let color = if active { palette.bottom } else { palette.text };
                    let label = if is_english {
                        layer.title_en
                    } else {
                        layer.title_zh
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(label).size(12.0).color(
                                if layer.supported {
                                    color
                                } else {
                                    palette.faint
                                },
                            ))
                            .fill(fill)
                            .corner_radius(egui::CornerRadius::same(12))
                            .min_size(Vec2::new(58.0, 28.0)),
                        )
                        .clicked()
                    {
                        if !layer.supported {
                            self.map_status = if is_english {
                                format!("{label} layer has no data adapter")
                            } else {
                                format!("{label}图层暂无数据适配器")
                            };
                            self.map_quality = DataQuality::Unavailable;
                            self.map_updated_at = None;
                            return;
                        }
                        self.map_layer = index;
                        self.map_generation += 1;
                        self.loading_map = false;
                        self.map_texture = None;
                        self.map_size = None;
                        self.map_frame_count = None;
                        self.map_frame_index = 0;
                        self.map_quality = DataQuality::Unavailable;
                        self.map_updated_at = None;
                        self.map_detail = None;
                        self.map_track.clear();
                        self.map_playing = false;
                        self.map_next_frame_at = None;
                        self.map_status = if is_english {
                            "Tap Refresh layer to load".into()
                        } else {
                            "点「刷新图层」加载".into()
                        };
                    }
                }
            });
            ui.add_space(8.0);

            if let (Some(texture), Some(size)) = (&self.map_texture, self.map_size) {
                let width = ui.available_width();
                let aspect = size[1] as f32 / size[0].max(1) as f32;
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(width, width * aspect), egui::Sense::hover());
                // 雷达和卫星图是带透明通道的叠加层，铺一层深色底衬才看得清
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(12),
                    Color32::from_rgba_unmultiplied(8, 18, 28, 200),
                );
                ui.painter().image(
                    texture.id(),
                    rect.shrink(3.0),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 132.0),
                    Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label(
                            RichText::new(if is_english {
                                "Precipitation radar and satellite\nshow the latest PAGASA frame"
                            } else {
                                "降水雷达与卫星图层\n显示 PAGASA 最新一帧"
                            })
                            .size(12.0)
                            .color(palette.faint),
                        );
                    },
                );
            }

            if let Some(count) = self.map_frame_count.filter(|count| *count > 1) {
                ui.add_space(6.0);
                let max = count.saturating_sub(1) as f32;
                let mut selected = self.map_frame_index as f32;
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if is_english { "Timeline" } else { "时间轴" })
                            .size(11.0)
                            .color(palette.faint),
                    );
                    changed = ui
                        .add(egui::Slider::new(&mut selected, 0.0..=max).show_value(false))
                        .changed();
                    ui.label(
                        RichText::new(format!("{}/{}", selected.round() as u32 + 1, count))
                            .size(11.0)
                            .color(palette.dim),
                    );
                });
                if changed {
                    self.map_frame_index = selected.round().clamp(0.0, max) as u32;
                    let ctx = ui.ctx().clone();
                    self.refresh_map(&ctx);
                }
            }

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if widgets::pill_button(
                    ui,
                    if is_english {
                        "Refresh"
                    } else {
                        "刷新图层"
                    },
                    palette,
                )
                .clicked()
                {
                    let ctx = ui.ctx().clone();
                    self.refresh_map(&ctx);
                }
                if self.map_frame_count.is_some_and(|count| count > 1) {
                    let label = if self.map_playing {
                        if is_english {
                            "Pause"
                        } else {
                            "暂停"
                        }
                    } else if is_english {
                        "Play"
                    } else {
                        "播放"
                    };
                    if widgets::pill_button(ui, label, palette).clicked() {
                        self.map_playing = !self.map_playing;
                        self.map_next_frame_at = Some(Instant::now());
                    }
                }
                #[cfg(target_os = "android")]
                if widgets::pill_button(
                    ui,
                    if is_english {
                        "Official map"
                    } else {
                        "官方地图"
                    },
                    palette,
                )
                .clicked()
                {
                    crate::open_official_map(OFFICIAL_MAP_URL);
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new(&self.map_status)
                    .size(11.0)
                    .color(palette.dim),
            );
            widgets::provenance_line(
                ui,
                palette,
                &self.map_source,
                self.map_updated_at.as_deref(),
                self.map_quality,
            );
        });

        if let Some(detail) = &self.map_detail {
            ui.add_space(10.0);
            theme::card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::card_header_public(
                    ui,
                    palette,
                    if is_english {
                        "Typhoon detail"
                    } else {
                        "台风路径摘要"
                    },
                    "PAGASA",
                );
                ui.add_space(4.0);
                let text = if is_english {
                    detail.clone()
                } else if let Some(rest) = detail.strip_prefix("Cyclone ") {
                    format!("台风 {rest}")
                } else {
                    detail.clone()
                };
                ui.label(RichText::new(text).size(12.0).color(palette.text));
            });
        }

        if !self.map_track.is_empty() {
            ui.add_space(10.0);
            widgets::typhoon_track(ui, &self.map_track, palette, is_english);
        }
    }

    fn alerts_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(if self.language.is_english() {
                        format!("{} alerts", self.dashboard.alerts.len())
                    } else {
                        format!("{} 条", self.dashboard.alerts.len())
                    })
                    .size(12.0)
                    .color(palette.faint),
                );
            });
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new(if self.language.is_english() {
                "Showing alerts for the selected place; nationwide alerts are kept"
            } else {
                "已按当前地点筛选预警，全国性预警仍会保留"
            })
            .size(11.0)
            .color(palette.faint),
        );
        ui.add_space(6.0);

        if self.dashboard.alerts.is_empty() {
            theme::card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    RichText::new(if self.language.is_english() {
                        "No public alerts"
                    } else {
                        "当前没有公开预警"
                    })
                    .size(13.0)
                    .color(palette.dim),
                );
            });
        }
        for alert in &self.dashboard.alerts {
            widgets::alert_card(ui, alert, palette, self.language.is_english());
            ui.add_space(8.0);
        }

        ui.add_space(4.0);
        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                if self.language.is_english() {
                    "Local notifications"
                } else {
                    "本地提醒"
                },
                palette,
            );
            ui.label(
                RichText::new(if self.language.is_english() {
                    format!("Status: {}", self.notification_status)
                } else {
                    format!("状态：{}", self.notification_status)
                })
                .size(12.0)
                .color(palette.dim),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                #[cfg(target_os = "android")]
                {
                    if widgets::pill_button(
                        ui,
                        if self.language.is_english() {
                            "Allow notifications"
                        } else {
                            "申请通知权限"
                        },
                        palette,
                    )
                    .clicked()
                    {
                        crate::request_notification_permission();
                        self.notification_status = if self.language.is_english() {
                            "Permission request started".into()
                        } else {
                            "已发起权限申请".into()
                        };
                    }
                    if widgets::pill_button(
                        ui,
                        if self.language.is_english() {
                            "Send test"
                        } else {
                            "发送测试通知"
                        },
                        palette,
                    )
                    .clicked()
                    {
                        crate::post_test_notification();
                        self.notification_status = if self.language.is_english() {
                            "Test notification sent".into()
                        } else {
                            "已发送测试通知".into()
                        };
                    }
                    if widgets::pill_button(
                        ui,
                        if self.language.is_english() {
                            "Start 30-min alerts"
                        } else {
                            "开启 30 分钟提醒"
                        },
                        palette,
                    )
                    .clicked()
                    {
                        self.notification_status = if crate::start_local_notification_scheduler() {
                            if self.language.is_english() {
                                "Every 30 minutes".into()
                            } else {
                                "每 30 分钟提醒一次".into()
                            }
                        } else {
                            if self.language.is_english() {
                                "Already running".into()
                            } else {
                                "提醒已在运行".into()
                            }
                        };
                        crate::write_preference("notifications_enabled", "1");
                    }
                    if widgets::pill_button(
                        ui,
                        if self.language.is_english() {
                            "Stop alerts"
                        } else {
                            "停止提醒"
                        },
                        palette,
                    )
                    .clicked()
                    {
                        crate::stop_local_notification_scheduler();
                        crate::write_preference("notifications_enabled", "0");
                        self.notification_status = if self.language.is_english() {
                            "Stopped".into()
                        } else {
                            "已停止".into()
                        };
                    }
                }
                #[cfg(not(target_os = "android"))]
                ui.label(
                    RichText::new(if self.language.is_english() {
                        "Notifications are available on Android"
                    } else {
                        "通知仅在 Android 上可用"
                    })
                    .size(11.0)
                    .color(palette.faint),
                );
            });
        });
    }

    // -------------------------------------------------------------- settings

    fn profile_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let button = if self.language.is_english() {
                    "中文"
                } else {
                    "English"
                };
                if widgets::pill_button(ui, button, palette).clicked() {
                    self.language = if self.language.is_english() {
                        UiLanguage::Chinese
                    } else {
                        UiLanguage::English
                    };
                    crate::write_preference(
                        "language",
                        if self.language.is_english() {
                            "en"
                        } else {
                            "zh"
                        },
                    );
                    self.map_status = if self.language.is_english() {
                        "Choose a layer, then refresh".into()
                    } else {
                        "选择图层后点刷新".into()
                    };
                    let enabled = crate::read_preference("notifications_enabled")
                        .as_deref()
                        .is_some_and(|value| value == "1");
                    self.notification_status = if enabled {
                        if self.language.is_english() {
                            "Every 30 minutes".into()
                        } else {
                            "每 30 分钟提醒一次".into()
                        }
                    } else if self.language.is_english() {
                        "Off".into()
                    } else {
                        "未启用".into()
                    };
                }
            });
        });
        ui.label(
            RichText::new(if self.language.is_english() {
                "Language / 语言"
            } else {
                "语言 · 可随时切换中文或 English"
            })
            .size(11.0)
            .color(palette.faint),
        );
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let is_english = self.language.is_english();
            for (index, label) in [
                (
                    2usize,
                    if is_english {
                        "Weather map"
                    } else {
                        "天气地图"
                    },
                ),
                (
                    3usize,
                    if is_english {
                        "Alerts & reminders"
                    } else {
                        "预警与提醒"
                    },
                ),
            ] {
                let response = ui
                    .horizontal(|ui| {
                        ui.label(RichText::new(label).size(13.0).color(palette.text));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(">").size(14.0).color(palette.faint));
                        });
                    })
                    .response;
                if response.interact(Sense::click()).clicked() {
                    self.tab = index;
                }
                ui.add_space(6.0);
            }
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                setting_text(
                    "runtime_status",
                    self.language.is_english(),
                    cfg!(target_os = "android"),
                ),
                palette,
            );
            let cached = if self.cached {
                setting_text(
                    "available",
                    self.language.is_english(),
                    cfg!(target_os = "android"),
                )
            } else {
                setting_text(
                    "unavailable",
                    self.language.is_english(),
                    cfg!(target_os = "android"),
                )
            };
            let permission = setting_text(
                "permission_android",
                self.language.is_english(),
                cfg!(target_os = "android"),
            );
            for (label, value) in [
                (
                    setting_text(
                        "weather_cache",
                        self.language.is_english(),
                        cfg!(target_os = "android"),
                    ),
                    cached,
                ),
                (
                    setting_text(
                        "notifications",
                        self.language.is_english(),
                        cfg!(target_os = "android"),
                    ),
                    self.notification_status.as_str(),
                ),
                (
                    setting_text(
                        "permissions",
                        self.language.is_english(),
                        cfg!(target_os = "android"),
                    ),
                    permission,
                ),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(12.0).color(palette.dim));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(value).size(11.0).color(palette.faint));
                    });
                });
            }
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                if self.language.is_english() {
                    "Current location"
                } else {
                    "当前地点"
                },
                palette,
            );
            ui.label(
                RichText::new(&self.dashboard.place.name)
                    .size(19.0)
                    .strong()
                    .color(palette.text),
            );
            ui.label(
                RichText::new(format!(
                    "{} · {:.3}, {:.3}",
                    self.dashboard.place.region,
                    self.dashboard.place.latitude,
                    self.dashboard.place.longitude
                ))
                .size(11.0)
                .color(palette.faint),
            );
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                if self.language.is_english() {
                    "Search location"
                } else {
                    "搜索地点"
                },
                palette,
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 74.0, 32.0],
                    egui::TextEdit::singleline(&mut self.search_query).hint_text(
                        if self.language.is_english() {
                            "Enter a city in English or Chinese"
                        } else {
                            "输入城市名（支持中文、英文或拼音）"
                        },
                    ),
                );
                if self.search_focus {
                    response.request_focus();
                    self.search_focus = false;
                }
                if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    let ctx = ui.ctx().clone();
                    self.search(&ctx);
                }
                if widgets::pill_button(
                    ui,
                    if self.language.is_english() {
                        "Search"
                    } else {
                        "搜索"
                    },
                    palette,
                )
                .clicked()
                {
                    let ctx = ui.ctx().clone();
                    self.search(&ctx);
                }
            });
            if self.searching {
                widgets::loading_row(
                    ui,
                    palette,
                    if self.language.is_english() {
                        "Searching…"
                    } else {
                        "正在搜索…"
                    },
                );
            }
            if self.search_attempted && !self.searching && self.search_results.is_empty() {
                ui.label(
                    RichText::new(if self.language.is_english() {
                        "No matching Philippine location was returned. Try an English or Chinese city name."
                    } else {
                        "没有找到可用地点。当前数据源覆盖菲律宾城市，请尝试中文、英文或拼音名称。"
                    })
                    .size(11.0)
                    .color(palette.faint),
                );
            }
            for place in self.search_results.clone() {
                if ui
                    .add_sized(
                        [ui.available_width(), 32.0],
                        egui::Button::new(
                            RichText::new(format!("{} · {}", place.name, place.region))
                                .size(13.0)
                                .color(palette.text),
                        )
                        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
                    )
                    .clicked()
                {
                    self.place_generation += 1;
                    self.loading_weather = false;
                    self.loading_air = false;
                    self.air = None;
                    self.air_status = None;
                    self.air_history.clear();
                    self.cached = false;
                    self.dashboard = Dashboard::empty(place);
                    self.tab = 0;
                    crate::write_preference("place", &encode_place(&self.dashboard.place));
                    self.search_results.clear();
                    self.search_attempted = false;
                    self.status = if self.language.is_english() {
                        "Location changed; refreshing…".into()
                    } else {
                        "已切换地点，正在刷新…".into()
                    };
                    let ctx = ui.ctx().clone();
                    self.refresh_weather(&ctx);
                    self.refresh_air(&ctx);
                }
            }
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                if self.language.is_english() {
                    "Data sources"
                } else {
                    "数据来源"
                },
                palette,
            );
            ui.add_space(4.0);
            let rows = if self.language.is_english() {
                [
                    ("Observation", "PAGASA automatic station · official"),
                    ("Forecast", "PAGASA nowcast · official"),
                    ("Air quality", "DENR-EMB report · official"),
                    ("Sun & UV", "Open-Meteo model · forecast"),
                    ("Lifestyle", "Local rules · reference only"),
                    ("Offline data", "Built-in sample"),
                ]
            } else {
                [
                    ("实时观测", "PAGASA 自动站 · 官方观测"),
                    ("天气预报", "PAGASA nowcast · 官方预报"),
                    ("空气质量", "DENR-EMB 报表 · 官方"),
                    ("日照与紫外线", "Open-Meteo 模型 · 预报"),
                    ("生活指数", "本地规则 · 仅供参考"),
                    ("离线数据", "应用内置样例"),
                ]
            };
            for (title, detail) in rows {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(12.0).color(palette.dim));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(detail).size(11.0).color(palette.faint));
                    });
                });
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(if self.language.is_english() {
                    "Values marked ≈ or “local estimate” are computed on this device, not official readings."
                } else {
                    "带 ≈ 或标注「本地参考」的数值由本机推算，不是官方观测。"
                })
                .size(10.0)
                .color(palette.faint),
            );
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(
                ui,
                if self.language.is_english() {
                    "About"
                } else {
                    "关于"
                },
                palette,
            );
            ui.add_space(4.0);
            let source = match (self.dashboard.source.clone(), self.language.is_english()) {
                (DataSource::Unavailable, true) => "No data",
                (DataSource::Unavailable, false) => "暂无数据",
                (DataSource::PublicApi, true) => "public API",
                (DataSource::OfflineSample, true) => "offline sample",
                (DataSource::PublicApi, false) => "公开接口",
                (DataSource::OfflineSample, false) => "离线样例",
            };
            ui.label(
                RichText::new(if self.language.is_english() {
                    format!("Current data: {source}")
                } else {
                    format!("当前数据：{source}")
                })
                .size(12.0)
                .color(palette.dim),
            );
            ui.label(
                RichText::new(if self.language.is_english() {
                    format!(
                        "Public service version: {}",
                        self.version.as_deref().unwrap_or("unknown")
                    )
                } else {
                    format!(
                        "公开服务版本：{}",
                        self.version.as_deref().unwrap_or("未知")
                    )
                })
                .size(11.0)
                .color(palette.faint),
            );
            ui.label(
                RichText::new(if self.language.is_english() {
                    "Independent implementation · Rust + egui · built for GrapheneOS"
                } else {
                    "独立实现 · Rust + egui · 适用于 GrapheneOS"
                })
                .size(11.0)
                .color(palette.faint),
            );
            ui.label(
                RichText::new(if self.language.is_english() {
                    "Weather and air-quality data belong to PAGASA and DENR-EMB."
                } else {
                    "数据版权归 PAGASA 与 DENR-EMB 所有。"
                })
                .size(10.0)
                .color(palette.faint),
            );
        });
    }

    /// 首页的地图入口卡，点击进入地图页。
    fn map_entry(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let is_english = self.language.is_english();
        let response = ui
            .scope(|ui| {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(if is_english {
                                    "Weather map"
                                } else {
                                    "天气地图"
                                })
                                .size(15.0)
                                .color(palette.text),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(if is_english {
                                    "Precipitation radar · satellite · typhoon track"
                                } else {
                                    "降水雷达 · 卫星 · 台风路径"
                                })
                                .size(11.0)
                                .color(palette.faint),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(">").size(18.0).color(palette.dim));
                        });
                    });
                });
            })
            .response;
        if response.interact(Sense::click()).clicked() {
            self.tab = 2;
        }
    }
    fn footer(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.add_space(4.0);
        let source = match (self.dashboard.source.clone(), self.language.is_english()) {
            (DataSource::Unavailable, true) => "No data",
            (DataSource::Unavailable, false) => "暂无数据",
            (DataSource::PublicApi, true) => "PAGASA public API",
            (DataSource::OfflineSample, true) => "Offline sample",
            (DataSource::PublicApi, false) => "PAGASA 公开接口",
            (DataSource::OfflineSample, false) => "离线样例",
        };
        let version = self
            .version
            .as_deref()
            .map(|value| {
                if self.language.is_english() {
                    format!(" · API {value}")
                } else {
                    format!(" · 接口 {value}")
                }
            })
            .unwrap_or_default();
        widgets::source_badge(
            ui,
            palette,
            &if self.language.is_english() {
                format!("Source {source}{version} · {}", self.status)
            } else {
                format!("数据来源 {source}{version} · {}", self.status)
            },
        );
        ui.add_space(6.0);
    }
}
impl PanahonApp {
    /// 顶部栏：首页显示菜单 / 城市 / 搜索 / 更多，子页显示返回 / 标题。
    ///
    /// 用常规布局分配交互区域，避免手算矩形与绘制位置脱节。
    fn top_bar(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let is_english = self.language.is_english();
        let stroke = egui::Stroke::new(2.0_f32, palette.text);
        let on_home = self.tab == 0;

        ui.horizontal_centered(|ui| {
            let (left_rect, left) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());
            let center = left_rect.center();
            if on_home {
                for row in 0..3 {
                    let y = center.y - 7.0 + row as f32 * 7.0;
                    ui.painter().line_segment(
                        [Pos2::new(center.x - 9.0, y), Pos2::new(center.x + 9.0, y)],
                        stroke,
                    );
                }
            } else {
                ui.painter().line_segment(
                    [
                        Pos2::new(center.x + 4.0, center.y - 8.0),
                        Pos2::new(center.x - 5.0, center.y),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(center.x - 5.0, center.y),
                        Pos2::new(center.x + 4.0, center.y + 8.0),
                    ],
                    stroke,
                );
            }
            if left.clicked() {
                if !on_home {
                    self.tab = 0;
                }
            }

            if on_home {
                egui::Popup::menu(&left).width(190.0).show(|ui| {
                    ui.label(
                        RichText::new(if is_english { "Weather" } else { "天气" })
                            .size(11.0)
                            .color(palette.faint),
                    );
                    if ui
                        .button(if is_english {
                            "City management"
                        } else {
                            "城市管理"
                        })
                        .clicked()
                    {
                        self.tab = 4;
                        self.search_focus = true;
                        self.search_attempted = false;
                    }
                    if ui
                        .button(if is_english {
                            "Weather map"
                        } else {
                            "天气地图"
                        })
                        .clicked()
                    {
                        self.tab = 2;
                    }
                    if ui
                        .button(if is_english { "Alerts" } else { "预警" })
                        .clicked()
                    {
                        self.tab = 3;
                    }
                    if ui
                        .button(if is_english { "Settings" } else { "设置" })
                        .clicked()
                    {
                        self.tab = 4;
                    }
                });
            }

            let title = match self.tab {
                0 => self.dashboard.place.name.clone(),
                1 => {
                    if is_english {
                        "Air quality".into()
                    } else {
                        "空气质量".into()
                    }
                }
                2 => {
                    if is_english {
                        "Weather map".into()
                    } else {
                        "天气地图".into()
                    }
                }
                3 => {
                    if is_english {
                        "Alerts".into()
                    } else {
                        "预警".into()
                    }
                }
                _ => {
                    if is_english {
                        "Settings".into()
                    } else {
                        "设置".into()
                    }
                }
            };

            let right_width = if on_home { 80.0 } else { 40.0 };
            let spare = (ui.available_width() - right_width).max(80.0);
            let (mid, _) = ui.allocate_exact_size(Vec2::new(spare, 40.0), Sense::hover());
            ui.painter().text(
                mid.center(),
                Align2::CENTER_CENTER,
                title,
                FontId::proportional(17.0),
                palette.text,
            );

            let (right_rect, right) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());
            if on_home {
                let center = right_rect.center();
                ui.painter()
                    .circle_stroke(center + Vec2::new(-2.0, -2.0), 7.0, stroke);
                ui.painter().line_segment(
                    [center + Vec2::new(3.0, 3.0), center + Vec2::new(9.0, 9.0)],
                    stroke,
                );
                if right.clicked() {
                    self.tab = 4;
                    self.search_results.clear();
                    self.search_focus = true;
                    self.search_attempted = false;
                }

                let (more_rect, more) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());
                for dx in [-3.0_f32, 3.0] {
                    for dy in [-3.0_f32, 3.0] {
                        ui.painter().circle_filled(
                            more_rect.center() + Vec2::new(dx, dy),
                            1.8,
                            palette.text,
                        );
                    }
                }
                egui::Popup::menu(&more).width(190.0).show(|ui| {
                    if ui
                        .button(if is_english {
                            "Refresh weather"
                        } else {
                            "刷新天气"
                        })
                        .clicked()
                    {
                        let ctx = ui.ctx().clone();
                        self.refresh_weather(&ctx);
                        self.refresh_air(&ctx);
                    }
                    if ui
                        .button(if is_english {
                            "Weather map"
                        } else {
                            "天气地图"
                        })
                        .clicked()
                    {
                        self.tab = 2;
                    }
                    if ui
                        .button(if is_english { "Alerts" } else { "预警" })
                        .clicked()
                    {
                        self.tab = 3;
                    }
                    if ui
                        .button(if is_english { "Settings" } else { "设置" })
                        .clicked()
                    {
                        self.tab = 4;
                    }
                });
            } else {
                let _ = right;
            }
        });
    }
}
impl eframe::App for PanahonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        self.refresh_insets(ctx);
        self.tick_map_playback(ctx);
        widgets::set_language(ctx, self.language.is_english());
        let palette = self.palette();
        let (safe_top, safe_bottom) = self
            .insets_points
            .map(|(top, bottom)| (top, bottom))
            .unwrap_or((theme::safe_top(), 0.0));

        // Sky gradient goes on the background layer, behind every panel.
        let screen = ctx.content_rect();
        theme::paint_background(
            &ctx.layer_painter(egui::LayerId::background()),
            screen,
            &palette,
        );

        egui::TopBottomPanel::top("top-bar")
            .exact_height(safe_top + 52.0)
            .frame(
                egui::Frame::NONE
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.add_space(safe_top);
                self.top_bar(ui, &palette);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::symmetric(theme::CARD_MARGIN as i8, 0)),
            )
            .show(ctx, |ui| {
                let area = egui::ScrollArea::vertical()
                    .id_salt(("page", self.tab, self.place_generation))
                    .auto_shrink([false, false]);
                // 桌面截图时可用 PANAHON_SCROLL 固定滚动位置
                let area = match std::env::var("PANAHON_SCROLL")
                    .ok()
                    .and_then(|value| value.parse::<f32>().ok())
                {
                    Some(offset) => area.vertical_scroll_offset(offset),
                    None => area,
                };
                area.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    match self.tab {
                        0 => self.weather_page(ui, &palette),
                        1 => self.air_page(ui, &palette),
                        2 => self.map_page(ui, &palette),
                        3 => self.alerts_page(ui, &palette),
                        _ => self.profile_page(ui, &palette),
                    }
                    ui.add_space(16.0 + safe_bottom);
                });
            });
    }
}
fn load_air_history() -> Vec<(String, u16)> {
    crate::read_preference("air_history")
        .unwrap_or_default()
        .split(';')
        .filter_map(|entry| {
            let (at, aqi) = entry.rsplit_once('|')?;
            Some((at.to_owned(), aqi.parse().ok()?))
        })
        .collect()
}

fn install_cjk_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_cjk".into(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansCJKsc-Subset.otf")).into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "noto_sans_cjk".into());
    }
    ctx.set_fonts(fonts);
}

/// Keep the small helper used by the map tab honest about its tuning.
#[allow(dead_code)]
fn _icon_size_hint() -> FontId {
    FontId::proportional(12.0)
}
