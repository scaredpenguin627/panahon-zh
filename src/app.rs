//! Application state and the five screens.
//!
//! Every network call runs on a worker thread and reports back through a
//! channel, so the UI keeps painting while a request is in flight.

use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Vec2};
use panahon_core::{
    fetch_air_quality_at, fetch_api_version, fetch_public_dashboard, fetch_remote_map_layer,
    search_places, AirQuality, ApiVersion, Dashboard, DataSource, Place, RemoteMapSnapshot,
};

#[cfg(target_os = "android")]
use panahon_core::OFFICIAL_MAP_URL;

use crate::theme::{self, Palette};
use crate::widgets;

const MAP_LAYERS: [(&str, &str); 5] = [
    ("radar", "雷达"),
    ("satellite", "卫星"),
    ("lightning", "闪电"),
    ("typhoon", "台风"),
    ("hazard", "灾害"),
];

enum Msg {
    Dashboard(Result<Dashboard, String>),
    Air(Result<AirQuality, String>),
    Version(Result<ApiVersion, String>),
    Places(Result<Vec<Place>, String>),
    Map(Box<(String, Result<RemoteMapSnapshot, String>)>),
}

pub struct PanahonApp {
    tab: usize,
    dashboard: Dashboard,
    status: String,
    loading_weather: bool,
    air: Option<AirQuality>,
    air_status: Option<String>,
    loading_air: bool,
    map_layer: usize,
    map_status: String,
    map_texture: Option<egui::TextureHandle>,
    map_size: Option<[usize; 2]>,
    loading_map: bool,
    search_query: String,
    search_results: Vec<Place>,
    searching: bool,
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
        let mut app = Self {
            tab: initial_tab,
            dashboard: Dashboard::sample(),
            status: "离线数据".into(),
            loading_weather: false,
            air: None,
            air_status: None,
            loading_air: false,
            map_layer: 0,
            map_status: "选择图层后点刷新".into(),
            map_texture: None,
            map_size: None,
            loading_map: false,
            search_query: String::new(),
            search_results: Vec::new(),
            searching: false,
            version: None,
            notification_status: "未启用".into(),
            tx,
            rx,
            insets_points: None,
            inset_attempts: 0,
        };
        app.refresh_weather(&cc.egui_ctx);
        app.refresh_air(&cc.egui_ctx);
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
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Dashboard(fetch_public_dashboard(&place)));
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
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Air(fetch_air_quality_at(latitude, longitude)));
            ctx.request_repaint();
        });
    }

    fn refresh_map(&mut self, ctx: &egui::Context) {
        if self.loading_map {
            return;
        }
        self.loading_map = true;
        self.map_status = "正在读取官方图层…".into();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let (key, title) = MAP_LAYERS[self.map_layer];
        let key = key.to_owned();
        let title = title.to_owned();
        std::thread::spawn(move || {
            let result = fetch_remote_map_layer(&key);
            let _ = tx.send(Msg::Map(Box::new((title, result))));
            ctx.request_repaint();
        });
    }

    fn search(&mut self, ctx: &egui::Context) {
        if self.searching || self.search_query.trim().chars().count() < 3 {
            return;
        }
        self.searching = true;
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
                Msg::Dashboard(Ok(dashboard)) => {
                    self.dashboard = dashboard;
                    self.status = "已更新".into();
                    self.loading_weather = false;
                }
                Msg::Dashboard(Err(error)) => {
                    self.status = format!("刷新失败：{error}");
                    self.loading_weather = false;
                }
                Msg::Air(Ok(air)) => {
                    self.air = Some(air);
                    self.air_status = None;
                    self.loading_air = false;
                }
                Msg::Air(Err(error)) => {
                    self.air_status = Some(error);
                    self.loading_air = false;
                }
                Msg::Version(Ok(version)) => self.version = Some(version.latest_version),
                Msg::Version(Err(_)) => {}
                Msg::Places(Ok(places)) => {
                    self.search_results = places;
                    self.searching = false;
                }
                Msg::Places(Err(error)) => {
                    self.status = format!("地点搜索失败：{error}");
                    self.searching = false;
                }
                Msg::Map(payload) => {
                    let (title, result) = *payload;
                    self.loading_map = false;
                    match result {
                        Ok(snapshot) => {
                            let latest =
                                snapshot.latest_at.clone().unwrap_or_else(|| "未知".into());
                            let count = snapshot
                                .frame_count
                                .map(|value| format!("{value} 帧"))
                                .or_else(|| snapshot.item_count.map(|value| format!("{value} 条")))
                                .unwrap_or_else(|| "已返回".into());
                            self.map_status = format!("{title}：{count}，最新 {latest}");
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
                            self.map_texture = None;
                            self.map_size = None;
                            self.map_status = format!("{title}读取失败：{error}");
                        }
                    }
                }
            }
        }
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
        let mut refresh = false;
        widgets::hero(ui, &self.dashboard, palette, &mut refresh);
        if refresh {
            let ctx = ui.ctx().clone();
            self.refresh_weather(&ctx);
            self.refresh_air(&ctx);
        }
        ui.add_space(14.0);

        widgets::hourly_strip(ui, &self.dashboard.hourly, palette);
        ui.add_space(10.0);

        widgets::daily_list(ui, &self.dashboard.forecast, palette);
        ui.add_space(10.0);

        match (&self.air, &self.air_status) {
            (Some(air), _) => widgets::aqi_summary(ui, air, palette),
            (None, Some(error)) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    theme::section_title(ui, "空气质量", palette);
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
                    theme::section_title(ui, "空气质量", palette);
                    widgets::loading_row(ui, palette, "正在读取 DENR-EMB 站点数据…");
                });
            }
        }
        ui.add_space(10.0);

        widgets::detail_grid(ui, &self.dashboard.current, palette);
        ui.add_space(10.0);

        widgets::life_indices(ui, &self.dashboard.life_indices(), palette);
        ui.add_space(10.0);

        if let Some(alert) = self.dashboard.alerts.first() {
            widgets::alert_card(ui, alert, palette);
            if self.dashboard.alerts.len() > 1 {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "另有 {} 条预警，见「预警」页",
                        self.dashboard.alerts.len() - 1
                    ))
                    .size(11.0)
                    .color(palette.faint),
                );
            }
            ui.add_space(10.0);
        }

        self.footer(ui, palette);
    }

    // ------------------------------------------------------------ air quality

    fn air_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("空气质量")
                    .size(24.0)
                    .strong()
                    .color(palette.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::pill_button(ui, "刷新", palette).clicked() {
                    let ctx = ui.ctx().clone();
                    self.refresh_air(&ctx);
                }
            });
        });
        ui.add_space(10.0);

        match (&self.air, &self.air_status) {
            (Some(air), _) => {
                let max = air.summary.max_aqi;
                theme::card_strong().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("AQI").size(11.0).color(palette.faint));
                            ui.label(
                                RichText::new(
                                    max.map(|value| value.to_string())
                                        .unwrap_or_else(|| "--".into()),
                                )
                                .size(52.0)
                                .strong()
                                .color(widgets::aqi_text_color(max)),
                            );
                        });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(
                                    air.summary
                                        .category
                                        .clone()
                                        .unwrap_or_else(|| "暂无".into()),
                                )
                                .size(18.0)
                                .strong()
                                .color(widgets::aqi_text_color(max)),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{} 个监测站 · 平均 {}",
                                    air.summary.station_count,
                                    air.summary
                                        .avg_aqi
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "--".into())
                                ))
                                .size(12.0)
                                .color(palette.dim),
                            );
                            if let Some(station) = &air.summary.max_station {
                                ui.label(
                                    RichText::new(format!("最高 {station}"))
                                        .size(12.0)
                                        .color(palette.dim),
                                );
                            }
                        });
                    });
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "{} · 数据时间 {}",
                            air.source.provider,
                            air.data_clock.as_deref().unwrap_or("未知")
                        ))
                        .size(10.0)
                        .color(palette.faint),
                    );
                });
                ui.add_space(10.0);

                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    theme::section_title(ui, "监测站（AQI 从高到低）", palette);
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
                        let color = widgets::aqi_text_color(station.aqi);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&station.name_zh)
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
                                    detail.push_str(&format!("首要 {pollutant}"));
                                }
                                ui.label(RichText::new(detail).size(10.0).color(palette.faint));
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(&station.category).size(11.0).color(color));
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
                        RichText::new("暂时读不到官方空气质量数据")
                            .size(15.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.add_space(4.0);
                    ui.label(RichText::new(error).size(11.0).color(palette.dim));
                });
            }
            (None, None) => {
                theme::card().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    widgets::loading_row(ui, palette, "正在读取 DENR-EMB 报表…");
                });
            }
        }
    }

    // -------------------------------------------------------------------- map

    fn map_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.label(
            RichText::new("天气地图")
                .size(24.0)
                .strong()
                .color(palette.text),
        );
        ui.label(
            RichText::new("PAGASA 公开图层，按需加载")
                .size(11.0)
                .color(palette.faint),
        );
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            for (index, (_, title)) in MAP_LAYERS.iter().enumerate() {
                let active = index == self.map_layer;
                let fill = if active {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 210)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 26)
                };
                let color = if active { palette.bottom } else { palette.text };
                if ui
                    .add(
                        egui::Button::new(RichText::new(*title).size(13.0).color(color))
                            .fill(fill)
                            .min_size(Vec2::new(62.0, 30.0)),
                    )
                    .clicked()
                {
                    self.map_layer = index;
                    self.map_texture = None;
                    self.map_size = None;
                    self.map_status = "点「刷新图层」加载".into();
                }
            }
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if widgets::pill_button(ui, "刷新图层", palette).clicked() {
                let ctx = ui.ctx().clone();
                self.refresh_map(&ctx);
            }
            #[cfg(target_os = "android")]
            if widgets::pill_button(ui, "打开官方地图", palette).clicked() {
                crate::open_official_map(OFFICIAL_MAP_URL);
            }
        });
        ui.label(
            RichText::new(&self.map_status)
                .size(11.0)
                .color(palette.dim),
        );
        ui.add_space(8.0);

        if let (Some(texture), Some(size)) = (&self.map_texture, self.map_size) {
            let width = ui.available_width();
            let aspect = size[1] as f32 / size[0].max(1) as f32;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(width, width * aspect), egui::Sense::hover());
            // 雷达和卫星图是带透明通道的叠加层，铺一层深色底衬才看得清
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(18),
                Color32::from_rgba_unmultiplied(8, 18, 28, 200),
            );
            ui.painter().image(
                texture.id(),
                rect.shrink(4.0),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            theme::card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 120.0),
                    Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label(
                            RichText::new("雷达与卫星图层\n显示 PAGASA 最新一帧")
                                .size(13.0)
                                .color(palette.faint),
                        );
                    },
                );
            });
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new("图像来自 PAGASA 公开图层，颜色和时间以官方地图为准。")
                .size(10.0)
                .color(palette.faint),
        );
    }

    // ----------------------------------------------------------------- alerts

    fn alerts_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("预警")
                    .size(24.0)
                    .strong()
                    .color(palette.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} 条", self.dashboard.alerts.len()))
                        .size(12.0)
                        .color(palette.faint),
                );
            });
        });
        ui.add_space(8.0);

        if self.dashboard.alerts.is_empty() {
            theme::card().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(
                    RichText::new("当前没有公开预警")
                        .size(13.0)
                        .color(palette.dim),
                );
            });
        }
        for alert in &self.dashboard.alerts {
            widgets::alert_card(ui, alert, palette);
            ui.add_space(8.0);
        }

        ui.add_space(4.0);
        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(ui, "本地提醒", palette);
            ui.label(
                RichText::new(format!("状态：{}", self.notification_status))
                    .size(12.0)
                    .color(palette.dim),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                #[cfg(target_os = "android")]
                {
                    if widgets::pill_button(ui, "申请通知权限", palette).clicked() {
                        crate::request_notification_permission();
                        self.notification_status = "已发起权限申请".into();
                    }
                    if widgets::pill_button(ui, "发送测试通知", palette).clicked() {
                        crate::post_test_notification();
                        self.notification_status = "已发送测试通知".into();
                    }
                    if widgets::pill_button(ui, "开启 30 分钟提醒", palette).clicked() {
                        self.notification_status = if crate::start_local_notification_scheduler() {
                            "每 30 分钟提醒一次".into()
                        } else {
                            "提醒已在运行".into()
                        };
                    }
                    if widgets::pill_button(ui, "停止提醒", palette).clicked() {
                        crate::stop_local_notification_scheduler();
                        self.notification_status = "已停止".into();
                    }
                }
                #[cfg(not(target_os = "android"))]
                ui.label(
                    RichText::new("通知仅在 Android 上可用")
                        .size(11.0)
                        .color(palette.faint),
                );
            });
        });
    }

    // -------------------------------------------------------------- settings

    fn profile_page(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.label(
            RichText::new("我的")
                .size(24.0)
                .strong()
                .color(palette.text),
        );
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(ui, "当前地点", palette);
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
            theme::section_title(ui, "搜索地点", palette);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 74.0, 32.0],
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("输入城市名，至少 3 个字母"),
                );
                if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    let ctx = ui.ctx().clone();
                    self.search(&ctx);
                }
                if widgets::pill_button(ui, "搜索", palette).clicked() {
                    let ctx = ui.ctx().clone();
                    self.search(&ctx);
                }
            });
            if self.searching {
                widgets::loading_row(ui, palette, "正在搜索…");
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
                    self.dashboard.place = place;
                    self.search_results.clear();
                    self.status = "已切换地点，正在刷新…".into();
                    let ctx = ui.ctx().clone();
                    self.refresh_weather(&ctx);
                    self.refresh_air(&ctx);
                }
            }
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(ui, "数据来源", palette);
            ui.add_space(4.0);
            let rows = [
                ("天气预报", "PAGASA PANaHON 公开接口"),
                ("空气质量", "DENR-EMB 实时空气质量报表"),
                ("雷达与卫星", "PAGASA 公开图层"),
                ("离线数据", "应用内置样例"),
            ];
            for (title, detail) in rows {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(12.0).color(palette.dim));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(detail).size(11.0).color(palette.faint));
                    });
                });
            }
        });
        ui.add_space(10.0);

        theme::card().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            theme::section_title(ui, "关于", palette);
            ui.add_space(4.0);
            let source = match self.dashboard.source {
                DataSource::PublicApi => "公开接口",
                DataSource::OfflineSample => "离线样例",
            };
            ui.label(
                RichText::new(format!("当前数据：{source}"))
                    .size(12.0)
                    .color(palette.dim),
            );
            ui.label(
                RichText::new(format!(
                    "公开服务版本：{}",
                    self.version.as_deref().unwrap_or("未知")
                ))
                .size(11.0)
                .color(palette.faint),
            );
            ui.label(
                RichText::new("独立实现 · Rust + egui · 适用于 GrapheneOS")
                    .size(11.0)
                    .color(palette.faint),
            );
            ui.label(
                RichText::new("数据版权归 PAGASA 与 DENR-EMB 所有。")
                    .size(10.0)
                    .color(palette.faint),
            );
        });
    }

    fn footer(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.add_space(4.0);
        let source = match self.dashboard.source {
            DataSource::PublicApi => "PAGASA 公开接口",
            DataSource::OfflineSample => "离线样例",
        };
        let version = self
            .version
            .as_deref()
            .map(|value| format!(" · 接口 {value}"))
            .unwrap_or_default();
        widgets::source_badge(
            ui,
            palette,
            &format!("数据来源 {source}{version} · {}", self.status),
        );
        ui.add_space(6.0);
    }
}
impl eframe::App for PanahonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        self.refresh_insets(ctx);
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

        let nav_height = if cfg!(target_os = "android") {
            58.0 + safe_bottom.max(18.0)
        } else {
            58.0 + safe_bottom
        };
        egui::TopBottomPanel::bottom("nav")
            .exact_height(nav_height)
            .frame(
                egui::Frame::NONE
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                if let Some(index) = widgets::bottom_nav(ui, self.tab, &palette, safe_bottom) {
                    self.tab = index;
                }
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(egui::Margin::symmetric(16, 0)),
            )
            .show(ctx, |ui| {
                ui.add_space(safe_top + 6.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match self.tab {
                            0 => self.weather_page(ui, &palette),
                            1 => self.air_page(ui, &palette),
                            2 => self.map_page(ui, &palette),
                            3 => self.alerts_page(ui, &palette),
                            _ => self.profile_page(ui, &palette),
                        }
                        ui.add_space(16.0);
                    });
            });
    }
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
