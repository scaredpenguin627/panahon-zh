# Huawei Weather APK UI evidence

This file records read-only evidence from `D:\\IDA PRO 9.5\\huawei-weather-14-0-15-301.apk` collected on 2026-11-09. It is an observation record, not a copy of Huawei source or assets.

## Package and implementation

`aapt dump badging` reports:

- package: `com.huawei.android.totemweather`
- version: `14.0.15.301` / versionCode `140015301`
- min SDK 28, target SDK 31, compile SDK 34
- native ABI present: `arm64-v8a`
- five DEX files: `classes.dex` through `classes5.dex`
- native libraries include `libyoga.so`, `libmmkv.so`, `libjslite.so`, `libTransform.so` and security/graphics libraries

The APK contains compiled Android XML layouts and Huawei custom views. This is a native Android resource/UI structure, not a Flutter-only layout. Relevant compiled resources include:

- `res/layout-zh-rCN/weather_single_city.xml`
- `res/layout-zh-rCN/weather_double_single_city_view.xml`
- `res/layout-land/home_city_weather.xml`
- `res/layout-land/city_weather_listview.xml`
- `res/layout-land/add_city_btn_layout.xml`

The APK also ships dedicated display fonts (`HW-digit-Regular.otf`, `HW-digit-Medium.otf`, `HW-digit-Bold.otf`) and a large set of weather/background assets. These assets are reference evidence only; the clean-room app continues to draw its own vectors.

## IDA Pro observation

IDA Pro 9.5 was used to open the APK through the `APK Android Package, multidex [dex.dll]` loader. The loaded main window identified the input as `huawei-weather-14-0-15-301.apk`; IDA created a temporary `.i64` analysis database. The IDA UI automation screenshot channel timed out while the large APK was still loading, so no proprietary code was copied or edited. The loader selection and loaded input title are authoritative evidence that the APK was opened through IDA rather than inferred only from web screenshots.

## Layout structure extracted with `aapt2 dump xmltree`

`weather_single_city.xml` has a `com.huawei.android.totemweather.view.WidgetHomeCityView` root with full width and centered content. Its main vertical stack contains:

1. A centered city `TextView`, single line, ellipsized, clickable, with a shadow.
2. A centered `DateTimeView` containing separate hour, date and lunar text views. The hour uses `sans-serif-thin`, no font padding, centered alignment, and a top margin.
3. A horizontal weather row containing a custom `IconImageView` and adjacent custom text views for the condition and temperature-related values.

`home_city_weather.xml` exposes the main full-screen composition and custom view names:

- `CityWeather`
- `CurrentTempView`
- `CurrentWeather`
- `IconImageView`
- `TempDiffColorView`
- `TextViewEMUI`
- `AlarmVerticalScrollView`
- `VoiceView`

The `city_weather_listview.xml` resource uses a custom `PullDownView`, `AntiFalseTouchListView`, and Huawei scrollbar wrapper. This confirms that the home screen is a vertically scrolling card/list composition with pull-down interaction, rather than a single fixed panel.

## Translation to the Rust/egui implementation

The evidence supports these layout decisions in `src/app.rs` and `src/widgets.rs`:

- Keep the city header centered and visually dominant.
- Keep the temperature/condition/icon block as one hero row.
- Use a single vertical scroll surface with cards for hourly, daily, AQI, observation, sun/UV, life indices and alerts.
- Keep date/hour text separate from the weather summary so labels do not collide with the icon.
- Use thin, high-contrast temperature typography and vector icons instead of copying Huawei assets.
- Keep map, alert and city-outlook entries as additional cards/pages rather than mixing them into the hero block.

The current app implements these structural patterns while using PAGASA/Open-Meteo data and its own Rust/egui rendering.
