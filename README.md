# 菲律宾天气（中文）

这是一个用 Rust 和 egui 编写的 Android 天气应用。数据来自 PAGASA、DENR-EMB、Open-Meteo 和 CAMS 的公开接口。

当前 Android 构建配置声明最低 SDK 29、目标 SDK 37，构建目标为 `aarch64-linux-android` 和 `x86_64-linux-android`。

## 当前已实现范围

当前版本包含以下功能：

- 天气首页和当前天气。
- 逐小时预报和五日预报。
- AQI 摘要和污染物状态。当前适配器可靠提供 PM2.5、PM10 的 AQI 子指数，其他污染物在数据缺失时显示不可用；监测站、趋势和历史属于当前版本排除范围。
- 降水雷达地图和时间轴。
- 预警列表、地点和全国范围过滤、有效期、官方原文和复制操作。
- 城市搜索和单地点选择。
- 中文和 English 切换。
- 来源、更新时间、数据质量、加载、缓存、错误和不可用状态。

## 数据来源和离线行为

| 功能 | 来源 | 说明 |
| --- | --- | --- |
| 实时天气、逐小时、五日预报 | PAGASA PANaHON | JSON 接口；显示来源、更新时间和质量 |
| 日出、日落、紫外线、能见度 | Open-Meteo | 位置模型预报 |
| 空气质量 | DENR-EMB | Power BI 公开报表；读不到时回退到 CAMS |
| AQI 趋势 | Open-Meteo / CAMS |  |
| 降水雷达 | PAGASA | 降水时间帧 |
| 预警 | PAGASA | 保留官方原文，并按当前地点和全国范围过滤 |

网络请求在后台线程执行。刷新失败时保留最近一次可信缓存，并显示失败原因。没有缓存时显示空状态。Android 另外保存地点、语言和最近的 AQI 历史记录。

## 桌面预览

需要 Rust 和 Cargo：

```powershell
cargo run --features "desktop network"
```

直接运行 `cargo build` 或带 `desktop` 的 Cargo 命令只构建 Rust 桌面目标和库，不生成 Android APK。没有 `DISPLAY` 或 `WAYLAND_DISPLAY` 时不能做桌面视觉验证。

## Android 构建

Android 构建需要以下材料：

- Android SDK，建议安装 API 35 和对应 build-tools。
- Android NDK，`build.ps1` 默认查找 `27.2.12479018`。
- JDK 和 `cargo-apk`。安装命令：`cargo install cargo-apk`。
- 一个本地 release keystore 及其密码。

在 PowerShell 中设置路径和签名材料：

```powershell
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\27.2.12479018"
$env:CARGO_APK_RELEASE_KEYSTORE = "keystore\panahon-release.jks"
$env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = "<密码>"
```

构建两个目标：

```powershell
.\build.ps1 -Network -Target aarch64-linux-android
.\build.ps1 -Network -Target x86_64-linux-android
```

`build.ps1` 会调用 `cargo apk build --release --lib`，并设置 NDK clang 和 16 KB 页面大小链接参数。成功后，本地 APK 位于 `target/release/apk/panahon-zh.apk`。该文件被 `.gitignore` 忽略。签名文件、密码和 APK 不进入 Git 仓库。

生成签名文件的示例：

```powershell
keytool -genkeypair -v -keystore keystore\panahon-release.jks -alias panahon `
  -keyalg RSA -keysize 2048 -validity 10000 `
  -storepass <密码> -keypass <密码> `
  -dname "CN=Panahon Zh, O=Panahon, C=PH"
```

## 发布状态

当前仓库没有 GitHub Release，也没有可下载的官方 APK 链接。APK 发布需要单独的签名、产物检查和上传流程。签名文件、密码和 APK 不提交到 Git 仓库。

## 兼容性状态

当前构建配置声明 API 29 及以上，目标架构为 `aarch64` 和 `x86_64`。32 位 ARM-only 设备不属于当前构建目标。

## 字体与布局限制

## 字体子集

## 许可
