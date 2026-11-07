#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Vec2};
use panahon_core::{
    fetch_api_version, fetch_public_dashboard, search_places, Dashboard, DataSource, ForecastDay,
    Place,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Places,
    Forecast,
    Map,
    Alerts,
}

impl Tab {
    const ALL: [Self; 4] = [Self::Places, Self::Forecast, Self::Map, Self::Alerts];

    fn title(self) -> &'static str {
        match self {
            Self::Places => "地点",
            Self::Forecast => "预报",
            Self::Map => "地图",
            Self::Alerts => "预警",
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
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_os = "android")]
        ui.add_space(28.0);
        ui.horizontal(|ui| {
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
        ui.horizontal(|ui| {
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

    fn map_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("天气地图");
        ui.label("地图数据来自公开服务。联网后可加载 PAGASA 雷达和观测图层。");
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 170.0),
                Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label(
                        RichText::new("雷达图层\n\n离线时不加载远程地图")
                            .size(20.0)
                            .color(Color32::from_rgb(76, 106, 121)),
                    );
                },
            );
        });
        ui.add_space(8.0);
        ui.label("数据源：PAGASA 官方公开页面。项目不内置 Google Play 或私有 SDK。");
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
                ui.label(RichText::new(&alert.issued_at).small().color(Color32::GRAY));
            });
            ui.add_space(6.0);
        }
        ui.label("通知默认关闭。后续可使用 Android 本地通知，不依赖 FCM。");
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

