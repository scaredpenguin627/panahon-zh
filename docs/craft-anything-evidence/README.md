# Craft Anything 分析记录

本目录保存 Craft Anything 的原始分析结果。分析对象是从 XAPK 解出的基础 APK 和 arm64 分包。

## 使用的命令

```powershell
craft-anything-cli --deep android-package-show --path "<base-apk>" --output-format json
craft-anything-cli --deep android-flutter-identify --path "<arm64-split-apk>" --output-format json
```

## 关键结果

- 包名为 `com.dostpagasa.panahon`，版本为 `1.1.4`，版本号为 `48`。
- 最低 SDK 为 `29`，目标 SDK 为 `36`。
- 基础 APK 有启动 Activity。
- arm64 分包包含 `libapp.so` 和 `libflutter.so`，识别结果为 Flutter，首选 ABI 为 `arm64-v8a`。
- 原清单包含 Firebase 和 Google Play 组件。Rust 重建没有加入 FCM、GSF、Play License 和设备注册依赖。

这些结果只用于确定功能范围和兼容性要求。Rust 工程没有复制原应用的 Flutter 二进制或打包资源。

## 华为天气 APK 的补充证据

`huawei-weather-14-0-15-301.apk` 由 Craft Anything `android-package-show` 读取，SHA-256 为 `b7c4aea9067feaa32445c72b75b02c143d6b9a0347259a4f5327e9925fd9980e`。结果显示包名 `com.huawei.android.totemweather`、版本 `14.0.15.301`、5 个 DEX、666 个资源条目、`arm64-v8a` 原生代码。`android-flutter-identify` 明确返回未发现 `libapp.so` 或 `libflutter.so`，因此本 APK 的页面证据应按原生 Android View/DEX 处理，不能按 Flutter AOT 页面推断。

Hex-Rays IDA MCP 以 Dalvik 处理器打开已生成的 `classes.dex.i64` 和 `classes3.dex.i64`。只导出名称和计数，不导出函数体：`classes.dex` 侧记录 49,069 个函数，`classes3.dex` 侧记录 52,685 个函数。后者命中了 `ShareCurrentTempView` 等天气自定义 View 符号；完整原始结果见 `huawei-ida-mcp.json` 和 `huawei-ui-ida-mcp.json`。这些名称只用来确认信息架构和控件类别，未复制实现。

在 Pixel 10a（GrapheneOS，1080×2424，420 dpi）上启动该 APK 会在 `WeatherApplication` 静态初始化阶段因缺少 `com.huawei.android.os.SystemPropertiesEx` 崩溃。`docs/huawei-weather-ida-evidence.md` 和 `docs/huawei-design-spec.md` 因此只采用 `aapt`、`apktool`、IDA 和 Craft Anything 的静态证据，不把启动画面当作可用的华为真机 UI 证据。

