# Rewrite context summary

## Objective

Rebuild the weather app in Rust/egui with a Huawei Weather-like information hierarchy while keeping a clean-room boundary. The Huawei APK is evidence for layout and behavior only; its code, fonts, images, shaders, and proprietary services are not copied.

## Reference APK evidence

- File: `D:\\IDA PRO 9.5\\huawei-weather-14-0-15-301.apk`
- Package: `com.huawei.android.totemweather`
- Version: `14.0.15.301`
- Native Android View app with five DEX files, AndroidX widgets, Huawei custom views, XML layouts, native arm64 libraries, custom fonts, weather animation assets and GLSL shaders.
- Main evidence: `docs/huawei-weather-evidence.md`.
- Relevant layouts: `weather_home.xml`, `weather_single_city.xml`, `weather_hourly_view`, `weather_day_forecast_layout_item_new`, `weather_air_quality_layout_new`, `weather_life_index`, `weather_sun_rise_set_new`, `weather_featured_map_layout`, and warning/feel/wind cards.

## Current implementation

- Repository: `D:\\IDA PRO 9.5\\panahon-zh`
- Rust/egui Android app with weather, hourly, five-day, AQI, map, alerts and settings tabs.
- PAGASA dashboard and alerts, DENR-EMB Power BI AQI, Open-Meteo sun/visibility/UV and AQI model trend.
- Data provenance: official observation, official forecast, locally estimated, unavailable.
- English/Chinese UI switch with Android persistence; selected place, AQI refresh history and notification switch persist through SharedPreferences.
- Radar/satellite map timeline supports manual selection and playback; typhoon map parses cyclone-track and draws projected path/wind-radius references.
- Font subset regenerated with `py -3.15 tools\\subset-font.py`; current source is `C:\\Windows\\Fonts\\NotoSansSC-VF.ttf`.
- Android build uses explicit NDK clang and `-z max-page-size=16384` flags.

## Verification baseline

- `cargo test --workspace --features "desktop network"`: 17 tests pass.
- `cargo check --target aarch64-linux-android --workspace --features network`: pass.
- Release APK built and installed on Pixel 10a; native ELF LOAD segments are 16 KB aligned.
- Release artifact: `target\\release\\apk\\panahon-zh.apk`.

## Remaining scope

- A true system-level AlarmManager receiver/service that continues after process death.
- Desktop/home-screen widgets.
- Huawei-like geographic basemap and full typhoon wind-cone overlay.
- In-app playback/cache for the PAGASA city-outlook video.

## Next evidence workflow

1. Preserve the APK/resource/aapt/IDA evidence under `docs/`.
2. Compare the current egui cards against the native layout hierarchy and actual device screenshots.
3. Implement only clean-room Rust/egui equivalents, preserving source and uncertainty labels.
4. Run font subsetting, workspace tests, Android checks, release build, ELF alignment check, and Pixel 10a smoke tests.
