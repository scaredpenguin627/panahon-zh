#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Vec2};
use panahon_core::{
    fetch_air_quality, fetch_api_version, fetch_public_dashboard, fetch_remote_map_layer,
    search_places, AirQuality, Dashboard, DataSource, ForecastDay, HourlyForecast, Place,
};

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "android")]
use std::thread;
#[cfg(target_os = "android")]
use std::time::Duration;

#[cfg(target_os = "android")]
static ANDROID_APP: OnceLock<winit::platform::android::activity::AndroidApp> = OnceLock::new();

#[cfg(target_os = "android")]
static NOTIFICATION_PERIODIC_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

#[cfg(target_os = "android")]
static NOTIFICATION_ONESHOT_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Places,
    Forecast,
    Map,
    Alerts,
    Services,
}

impl Tab {
    const ALL: [Self; 5] = [
        Self::Places,
        Self::Forecast,
        Self::Map,
        Self::Alerts,
        Self::Services,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Places => "地点",
            Self::Forecast => "预报",
            Self::Map => "地图",
            Self::Alerts => "预警",
            Self::Services => "服务",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MapLayer {
    Radar,
    Satellite,
    Lightning,
    Typhoon,
    Hazard,
}

impl MapLayer {
    const ALL: [Self; 5] = [
        Self::Radar,
        Self::Satellite,
        Self::Lightning,
        Self::Typhoon,
        Self::Hazard,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Radar => "雷达",
            Self::Satellite => "卫星",
            Self::Lightning => "闪电",
            Self::Typhoon => "台风",
            Self::Hazard => "灾害风险",
        }
    }

    fn api_key(self) -> &'static str {
        match self {
            Self::Radar => "radar",
            Self::Satellite => "satellite",
            Self::Lightning => "lightning",
            Self::Typhoon => "typhoon",
            Self::Hazard => "hazard",
        }
    }

    #[cfg(target_os = "android")]
    fn official_url(self) -> &'static str {
        match self {
            Self::Radar => "https://panahon.gov.ph/?trg=iframe&req=radar.rain-rate",
            Self::Satellite | Self::Lightning | Self::Typhoon | Self::Hazard => {
                panahon_core::OFFICIAL_MAP_URL
            }
        }
    }
}

pub struct PanahonApp {
    tab: Tab,
    dashboard: Dashboard,
    status: String,
    last_version: Option<String>,
    search_query: String,
    search_results: Vec<Place>,
    map_layer: MapLayer,
    map_status: String,
    map_texture: Option<egui::TextureHandle>,
    map_texture_size: Option<[usize; 2]>,
    air_quality: Option<AirQuality>,
    aqi_status: String,
    notification_status: String,
}

impl PanahonApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cjk_font(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.visuals.window_corner_radius = CornerRadius::same(18);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(12);
        cc.egui_ctx.set_style(style);
        Self {
            tab: Tab::Forecast,
            dashboard: Dashboard::sample(),
            status: "离线数据".into(),
            last_version: None,
            search_query: String::new(),
            search_results: Vec::new(),
            map_layer: MapLayer::Radar,
            map_status: "尚未加载远程图层".into(),
            map_texture: None,
            map_texture_size: None,
            air_quality: None,
            aqi_status: "尚未加载空气质量".into(),
            notification_status: "未启用".into(),
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_os = "android")]
        ui.add_space(28.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("PANaHON")
                    .size(26.0)
                    .strong()
                    .color(Color32::from_rgb(12, 91, 128)),
            );
            ui.label(
                RichText::new("中文天气")
                    .size(16.0)
                    .color(Color32::from_rgb(75, 95, 108)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("刷新").clicked() {
                    self.refresh_data();
                }
            });
        });
        ui.separator();
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for tab in Tab::ALL {
                let selected = self.tab == tab;
                let label = if selected {
                    RichText::new(tab.title()).strong().color(Color32::WHITE)
                } else {
                    RichText::new(tab.title()).color(Color32::from_rgb(40, 61, 73))
                };
                let fill = if selected {
                    Color32::from_rgb(12, 112, 145)
                } else {
                    Color32::from_rgb(225, 239, 243)
                };
                if ui
                    .add(
                        egui::Button::new(label)
                            .fill(fill)
                            .min_size(Vec2::new(76.0, 36.0)),
                    )
                    .clicked()
                {
                    self.tab = tab;
                }
            }
        });
        ui.add_space(8.0);
    }

    fn forecast_page(&mut self, ui: &mut egui::Ui) {
        let current = &self.dashboard.current;
        ui.group(|ui| {
            ui.label(
                RichText::new(&self.dashboard.place.name)
                    .size(20.0)
                    .strong(),
            );
            ui.label(RichText::new(&self.dashboard.place.region).color(Color32::GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{:.0} °C", current.temperature_c))
                        .size(52.0)
                        .strong(),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(&current.condition).size(18.0));
                    ui.label(format!(
                        "体感 {:.0} °C · 湿度 {}%",
                        current.feels_like_c, current.humidity_percent
                    ));
                    ui.label(format!(
                        "风速 {:.0} km/h · 风向 {}",
                        current.wind_kph, current.wind_direction
                    ));
                });
            });
            ui.label(
                RichText::new(format!("更新时间：{}", current.updated_at))
                    .small()
                    .color(Color32::GRAY),
            );
        });
        ui.add_space(8.0);
        ui.label(RichText::new("五日预报").size(18.0).strong());
        for day in &self.dashboard.forecast {
            forecast_row(ui, day);
        }
        ui.add_space(8.0);
        ui.label(RichText::new("逐小时预报").size(18.0).strong());
        if self.dashboard.hourly.is_empty() {
            ui.label("当前接口没有返回逐小时数据。");
        } else {
            for hour in &self.dashboard.hourly {
                hourly_row(ui, hour);
            }
        }
        ui.add_space(8.0);
        if ui.button("刷新天气数据").clicked() {
            self.refresh_data();
        }
        ui.add_space(8.0);
        source_badge(
            ui,
            &self.dashboard.source,
            &self.status,
            self.last_version.as_deref(),
        );
    }

    fn refresh_data(&mut self) {
        let place = self.dashboard.place.clone();
        if let Ok(version) = fetch_api_version() {
            self.last_version = Some(version.latest_version);
        }
        match fetch_public_dashboard(&place) {
            Ok(dashboard) => {
                self.dashboard = dashboard;
                self.status = "已更新公开天气数据".into();
            }
            Err(error) => self.status = format!("刷新失败，继续显示当前数据：{error}"),
        }
    }

    fn places_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("地点");
        ui.label("保存常用地点。切换地点后，点击“刷新”加载天气。");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [250.0, 32.0],
                egui::TextEdit::singleline(&mut self.search_query),
            );
            if ui.button("搜索").clicked() {
                match search_places(&self.search_query) {
                    Ok(results) => {
                        self.status = format!("找到 {} 个地点", results.len());
                        self.search_results = results;
                    }
                    Err(error) => self.status = format!("地点搜索失败：{error}"),
                }
            }
        });
        for place in self.search_results.clone() {
            if ui
                .button(format!("{} · {}", place.name, place.region))
                .clicked()
            {
                self.dashboard.place = place;
                self.status = "已切换地点；点击刷新加载天气".into();
            }
        }
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(RichText::new("当前地点").strong());
            ui.label(&self.dashboard.place.name);
            ui.label(format!(
                "坐标：{:.4}，{:.4}",
                self.dashboard.place.latitude, self.dashboard.place.longitude
            ));
            ui.label("定位权限仅在需要时申请。GrapheneOS 可以关闭应用的网络权限。");
        });
        ui.add_space(8.0);
        ui.label("示例地点");
        for place in ["宿务", "达沃", "碧瑶", "长滩岛", "香港"] {
            if ui.button(place).clicked() {
                self.status = format!("已选择 {place}。当前显示离线数据。");
            }
        }
    }

    fn refresh_map_layer(&mut self, ctx: &egui::Context) {
        let layer = self.map_layer;
        self.map_status = format!("正在读取 {} 远程图层……", layer.title());
        match fetch_remote_map_layer(layer.api_key()) {
            Ok(snapshot) => {
                let latest = snapshot.latest_at.as_deref().unwrap_or("未知时间");
                let count = snapshot
                    .frame_count
                    .map(|value| format!("{} 帧", value))
                    .or_else(|| snapshot.item_count.map(|value| format!("{} 条", value)))
                    .unwrap_or_else(|| "已返回".into());
                self.map_status = format!("{}：{}，最新 {}", layer.title(), count, latest);
                self.map_texture = None;
                self.map_texture_size = None;
                if let Some(bytes) = snapshot.image {
                    match image::load_from_memory(&bytes) {
                        Ok(decoded) => {
                            let rgba = decoded.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                rgba.as_flat_samples().as_slice(),
                            );
                            self.map_texture = Some(ctx.load_texture(
                                format!("panahon-map-{}", layer.api_key()),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            ));
                            self.map_texture_size = Some(size);
                        }
                        Err(error) => {
                            self.map_status =
                                format!("{} 元数据已更新，但图像无法解码：{error}", layer.title());
                        }
                    }
                }
            }
            Err(error) => {
                self.map_texture = None;
                self.map_texture_size = None;
                self.map_status = format!("远程图层读取失败：{error}");
            }
        }
    }

    fn map_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("天气地图");
        ui.label("选择图层后，应用会读取 PAGASA 的最新公开数据。");
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for layer in MapLayer::ALL {
                let selected = self.map_layer == layer;
                let fill = if selected {
                    Color32::from_rgb(12, 112, 145)
                } else {
                    Color32::from_rgb(225, 239, 243)
                };
                let color = if selected {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(40, 61, 73)
                };
                if ui
                    .add(egui::Button::new(RichText::new(layer.title()).color(color)).fill(fill))
                    .clicked()
                {
                    self.map_layer = layer;
                    self.map_status = "尚未加载远程图层".into();
                    self.map_texture = None;
                    self.map_texture_size = None;
                }
            }
        });
        ui.add_space(8.0);
        let layer_status = match self.map_layer {
            MapLayer::Radar => ("雷达图层", "读取 PAGASA 雷达降雨率时间轴和最新图像。"),
            MapLayer::Satellite => ("卫星图层", "读取 Himawari 红外卫星时间轴和最新图像。"),
            MapLayer::Lightning => ("闪电图层", "读取 PAGASA 最近 10 分钟的闪电点数据。"),
            MapLayer::Typhoon => ("台风图层", "读取当前台风轨迹和最新观测时间。"),
            MapLayer::Hazard => ("灾害风险图层", "读取 PAGASA 当前公开灾害预警数量。"),
        };
        ui.label(RichText::new(layer_status.0).strong());
        ui.label(layer_status.1);
        ui.horizontal_wrapped(|ui| {
            if ui.button("刷新远程图层").clicked() {
                self.refresh_map_layer(ui.ctx());
            }
            if ui.button("打开 PAGASA 官方交互地图").clicked() {
                #[cfg(target_os = "android")]
                open_official_map(self.map_layer.official_url());
                #[cfg(not(target_os = "android"))]
                ui.label("仅 Android 可用");
            }
        });
        ui.label(
            RichText::new(&self.map_status)
                .small()
                .color(Color32::from_rgb(58, 108, 83)),
        );
        if let (Some(texture), Some(size)) = (&self.map_texture, self.map_texture_size) {
            let width = ui.available_width().max(240.0);
            let aspect = size[1] as f32 / size[0].max(1) as f32;
            let height = (width * aspect).clamp(120.0, 260.0);
            ui.image((texture.id(), Vec2::new(width, height)));
            ui.label(
                RichText::new("图像来自 PAGASA 公开图层，颜色和时间以官方地图为准。")
                    .small()
                    .color(Color32::GRAY),
            );
        }
        ui.group(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 170.0),
                Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label(
                        RichText::new(
                            "交互地图\n\n雷达和卫星显示最新远程图像，其他图层显示数据数量",
                        )
                        .size(20.0)
                        .color(Color32::from_rgb(76, 106, 121)),
                    );
                },
            );
        });
        ui.add_space(8.0);
        ui.label("数据源：PAGASA 官方公开页面。应用只读取公开接口，不保存访问令牌。");
    }

    fn alerts_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("预警");
        ui.label("联网后刷新预警列表。");
        ui.add_space(8.0);
        for alert in &self.dashboard.alerts {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(222, 135, 41)));
                    ui.label(RichText::new(&alert.title).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(&alert.severity).color(Color32::from_rgb(170, 94, 34)),
                        );
                    });
                });
                ui.label(&alert.area);
                ui.label(&alert.detail);
                ui.label(
                    RichText::new(format!("防御建议：{}", alert.defense))
                        .color(Color32::from_rgb(192, 155, 96)),
                );
                ui.label(RichText::new(&alert.issued_at).small().color(Color32::GRAY));
            });
            ui.add_space(6.0);
        }
        ui.group(|ui| {
            ui.label(RichText::new("本地通知").strong());
            ui.label(format!("当前状态：{}", self.notification_status));
            ui.label("通知使用 Android 本地通道，不依赖 FCM。首次使用需要允许通知权限。");
            ui.horizontal(|ui| {
                if ui.button("请求通知权限").clicked() {
                    #[cfg(target_os = "android")]
                    {
                        request_notification_permission();
                        self.notification_status = "已发起权限请求".into();
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label("仅 Android 可用");
                }
                if ui.button("发送测试通知").clicked() {
                    #[cfg(target_os = "android")]
                    {
                        post_test_notification();
                        self.notification_status = "已发送测试通知".into();
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label("仅 Android 可用");
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button("启动本地调度").clicked() {
                    #[cfg(target_os = "android")]
                    {
                        if start_local_notification_scheduler() {
                            self.notification_status = "已启动，每 30 分钟发送一次本地提醒".into();
                        } else {
                            self.notification_status = "本地调度已经在运行".into();
                        }
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label("仅 Android 可用");
                }
                if ui.button("安排 1 分钟测试").clicked() {
                    #[cfg(target_os = "android")]
                    {
                        schedule_one_shot_notification();
                        self.notification_status = "已安排 1 分钟后发送本地通知".into();
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label("仅 Android 可用");
                }
                if ui.button("停止本地调度").clicked() {
                    #[cfg(target_os = "android")]
                    {
                        stop_local_notification_scheduler();
                        self.notification_status = "已停止本地调度".into();
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label("仅 Android 可用");
                }
            });
            ui.label(
                RichText::new("调度器在应用进程存活时运行；系统回收进程后，需要再次打开应用。")
                    .small()
                    .color(Color32::GRAY),
            );
        });
    }

    fn services_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("服务");
        ui.label("这些模块已预留界面边界。实时数据需要独立来源和许可。");
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("空气质量").strong());
                if ui.button("刷新空气质量").clicked() {
                    match fetch_air_quality() {
                        Ok(data) => {
                            self.aqi_status = format!(
                                "{} 个站点，平均 AQI {}，最差：{}",
                                data.summary.station_count,
                                data.summary
                                    .avg_aqi
                                    .map_or("暂无".into(), |value| value.to_string()),
                                data.summary.max_station.as_deref().unwrap_or("暂无")
                            );
                            self.air_quality = Some(data);
                        }
                        Err(error) => self.aqi_status = error,
                    }
                }
            });
            ui.label(&self.aqi_status);
            if let Some(data) = &self.air_quality {
                ui.label(format!(
                    "数据时间：{} · 来源：{}",
                    data.data_clock
                        .as_deref()
                        .or(data.updated_at.as_deref())
                        .unwrap_or("未知"),
                    data.source.provider
                ));
                ui.label("空气质量数据仅作信息参考，站点质量状态和更新时间以官方页面为准。");
                ui.collapsing("监测站明细（按 AQI 从高到低）", |ui| {
                    for station in data.stations.iter().take(8) {
                        ui.horizontal(|ui| {
                            ui.add_sized([150.0, 20.0], egui::Label::new(&station.name_zh));
                            ui.add_sized(
                                [72.0, 20.0],
                                egui::Label::new(format!(
                                    "AQI {}",
                                    station.aqi.map_or("暂无".into(), |value| value.to_string())
                                )),
                            );
                            ui.label(&station.category);
                        });
                    }
                });
            }
        });
        ui.add_space(8.0);
        let services = [
            ("旅游天气", "旅游区未来一周天气。", "公开接口待接入"),
            ("生活指数", "紫外线、穿衣、出行和户外活动建议。", "规则样例"),
        ];
        for (title, summary, status) in services {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(status)
                                .small()
                                .color(Color32::from_rgb(183, 141, 71)),
                        );
                    });
                });
                ui.label(summary);
            });
            ui.add_space(5.0);
        }
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

impl eframe::App for PanahonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.header(ui);
            self.tab_bar(ui);
            match self.tab {
                Tab::Places => self.places_page(ui),
                Tab::Forecast => self.forecast_page(ui),
                Tab::Map => self.map_page(ui),
                Tab::Alerts => self.alerts_page(ui),
                Tab::Services => self.services_page(ui),
            }
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("公开数据 · Rust · 适用于 GrapheneOS")
                        .small()
                        .color(Color32::GRAY),
                );
            });
        });
    }
}

fn forecast_row(ui: &mut egui::Ui, day: &ForecastDay) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [54.0, 20.0],
            egui::Label::new(RichText::new(&day.date).strong()),
        );
        ui.add_sized([110.0, 20.0], egui::Label::new(&day.summary));
        ui.add_sized(
            [70.0, 20.0],
            egui::Label::new(format!("{:.0} °C / {:.0} °C", day.low_c, day.high_c)),
        );
        ui.label(format!("降雨 {}%", day.rain_probability));
    });
}

fn hourly_row(ui: &mut egui::Ui, hour: &HourlyForecast) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [84.0, 20.0],
            egui::Label::new(RichText::new(&hour.time).strong()),
        );
        ui.add_sized([130.0, 20.0], egui::Label::new(&hour.summary));
        ui.add_sized(
            [70.0, 20.0],
            egui::Label::new(format!("{:.0} °C", hour.temperature_c)),
        );
        ui.label(format!("降雨 {}%", hour.rain_probability));
    });
}

fn source_badge(ui: &mut egui::Ui, source: &DataSource, status: &str, version: Option<&str>) {
    let label = match source {
        DataSource::OfflineSample => "数据来源：离线样例",
        DataSource::PublicApi => "数据来源：公开接口",
    };
    ui.separator();
    ui.label(RichText::new(label).color(Color32::from_rgb(58, 108, 83)));
    ui.label(RichText::new(status).small().color(Color32::GRAY));
    if let Some(version) = version {
        ui.label(
            RichText::new(format!("公开服务版本：{version}"))
                .small()
                .color(Color32::GRAY),
        );
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let _ = ANDROID_APP.set(app.clone());
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let result = eframe::run_native(
        "PANaHON 中文天气",
        options,
        Box::new(|cc| Ok(Box::new(PanahonApp::new(cc)))),
    );
    if let Err(error) = result {
        eprintln!("Android UI failed to start: {error}");
    }
}

#[cfg(target_os = "android")]
fn request_notification_permission() {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let permission = env.new_string("android.permission.POST_NOTIFICATIONS")?;
            let string_class = env.find_class(jni::jni_str!("java/lang/String"))?;
            let permissions =
                env.new_object_array(1, string_class, jni::objects::JObject::null())?;
            permissions.set_element(&env, 0, &permission)?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("requestPermissions"),
                jni::jni_sig!("([Ljava/lang/String;I)V"),
                &[
                    jni::objects::JValue::Object(&permissions),
                    jni::objects::JValue::Int(1001),
                ],
            )?;
            Ok(())
        });
    }));
}

#[cfg(target_os = "android")]
fn start_local_notification_scheduler() -> bool {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return false;
    };
    let slot = NOTIFICATION_PERIODIC_STOP.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().expect("notification scheduler mutex poisoned");
    if guard.is_some() {
        return false;
    }
    let stop = Arc::new(AtomicBool::new(false));
    *guard = Some(stop.clone());
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            if wait_for_notification(&stop, Duration::from_secs(30 * 60)) {
                break;
            }
            post_notification_with_app(
                app.clone(),
                "PANaHON 预警".into(),
                "本地调度已触发，请打开应用刷新 PAGASA 公开预警。".into(),
                2001,
            );
        }
    });
    true
}

#[cfg(target_os = "android")]
fn schedule_one_shot_notification() {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    let slot = NOTIFICATION_ONESHOT_STOP.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().expect("notification timer mutex poisoned");
    if let Some(previous) = guard.replace(Arc::new(AtomicBool::new(false))) {
        previous.store(true, Ordering::Relaxed);
    }
    let stop = guard.as_ref().expect("timer was just stored").clone();
    thread::spawn(move || {
        if !wait_for_notification(&stop, Duration::from_secs(60)) {
            post_notification_with_app(
                app,
                "PANaHON 预警".into(),
                "这是 1 分钟本地调度测试通知。".into(),
                2002,
            );
        }
    });
}

#[cfg(target_os = "android")]
fn stop_local_notification_scheduler() {
    if let Some(slot) = NOTIFICATION_PERIODIC_STOP.get() {
        if let Some(stop) = slot
            .lock()
            .expect("notification scheduler mutex poisoned")
            .take()
        {
            stop.store(true, Ordering::Relaxed);
        }
    }
    if let Some(slot) = NOTIFICATION_ONESHOT_STOP.get() {
        if let Some(stop) = slot
            .lock()
            .expect("notification timer mutex poisoned")
            .take()
        {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "android")]
fn wait_for_notification(stop: &AtomicBool, duration: Duration) -> bool {
    let mut elapsed = Duration::ZERO;
    while elapsed < duration {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        thread::sleep(Duration::from_secs(1));
        elapsed += Duration::from_secs(1);
    }
    stop.load(Ordering::Relaxed)
}

#[cfg(target_os = "android")]
fn post_test_notification() {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    post_notification_with_app(
        app,
        "PANaHON 预警".into(),
        "这是本地通知测试。".into(),
        1001,
    );
}

#[cfg(target_os = "android")]
fn post_notification_with_app(
    app: winit::platform::android::activity::AndroidApp,
    title: String,
    body: String,
    notification_id: i32,
) {
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let service_name = env.new_string("notification")?;
            let manager = env
                .call_method(
                    activity.as_ref(),
                    jni::jni_str!("getSystemService"),
                    jni::jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[jni::objects::JValue::Object(&service_name)],
                )?
                .l()?;
            let channel_id = env.new_string("panahon_alerts")?;
            let channel_name = env.new_string("PANaHON 预警")?;
            let channel = env.new_object(
                jni::jni_str!("android/app/NotificationChannel"),
                jni::jni_sig!("(Ljava/lang/String;Ljava/lang/CharSequence;I)V"),
                &[
                    jni::objects::JValue::Object(&channel_id),
                    jni::objects::JValue::Object(&channel_name),
                    jni::objects::JValue::Int(3),
                ],
            )?;
            env.call_method(
                &manager,
                jni::jni_str!("createNotificationChannel"),
                jni::jni_sig!("(Landroid/app/NotificationChannel;)V"),
                &[jni::objects::JValue::Object(&channel)],
            )?;
            let builder = env.new_object(
                jni::jni_str!("android/app/Notification$Builder"),
                jni::jni_sig!("(Landroid/content/Context;Ljava/lang/String;)V"),
                &[
                    jni::objects::JValue::Object(activity.as_ref()),
                    jni::objects::JValue::Object(&channel_id),
                ],
            )?;
            let title = env.new_string(title)?;
            let body = env.new_string(body)?;
            env.call_method(
                &builder,
                jni::jni_str!("setContentTitle"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Object(&title)],
            )?;
            env.call_method(
                &builder,
                jni::jni_str!("setContentText"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Object(&body)],
            )?;
            let icon = env
                .get_static_field(
                    jni::jni_str!("android/R$drawable"),
                    jni::jni_str!("sym_def_app_icon"),
                    jni::jni_sig!("I"),
                )?
                .i()?;
            env.call_method(
                &builder,
                jni::jni_str!("setSmallIcon"),
                jni::jni_sig!("(I)Landroid/app/Notification$Builder;"),
                &[jni::objects::JValue::Int(icon)],
            )?;
            let notification = env
                .call_method(
                    &builder,
                    jni::jni_str!("build"),
                    jni::jni_sig!("()Landroid/app/Notification;"),
                    &[],
                )?
                .l()?;
            env.call_method(
                &manager,
                jni::jni_str!("notify"),
                jni::jni_sig!("(ILandroid/app/Notification;)V"),
                &[
                    jni::objects::JValue::Int(notification_id),
                    jni::objects::JValue::Object(&notification),
                ],
            )?;
            Ok(())
        });
    }));
}

#[cfg(target_os = "android")]
fn open_official_map(url: &str) {
    let Some(app) = ANDROID_APP.get().cloned() else {
        return;
    };
    let url = url.to_owned();
    app.clone().run_on_java_main_thread(Box::new(move || {
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) };
        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let raw_activity = app.activity_as_ptr() as jni::sys::jobject;
            let activity = unsafe {
                env.as_cast_raw::<jni::refs::Global<jni::objects::JObject>>(&raw_activity)?
            };
            let action = env.new_string("android.intent.action.VIEW")?;
            let intent = env.new_object(
                jni::jni_str!("android/content/Intent"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[jni::objects::JValue::Object(&action)],
            )?;
            let url = env.new_string(url)?;
            let uri = env
                .call_static_method(
                    jni::jni_str!("android/net/Uri"),
                    jni::jni_str!("parse"),
                    jni::jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
                    &[jni::objects::JValue::Object(&url)],
                )?
                .l()?;
            env.call_method(
                &intent,
                jni::jni_str!("setData"),
                jni::jni_sig!("(Landroid/net/Uri;)Landroid/content/Intent;"),
                &[jni::objects::JValue::Object(&uri)],
            )?;
            env.call_method(
                activity.as_ref(),
                jni::jni_str!("startActivity"),
                jni::jni_sig!("(Landroid/content/Intent;)V"),
                &[jni::objects::JValue::Object(&intent)],
            )?;
            Ok(())
        });
    }));
}
