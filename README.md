# 菲律宾天气（中文）

这是一个用 Rust 和 egui 编写的 Android 天气应用。数据来自 PAGASA、DENR-EMB、Open-Meteo 和 CAMS 的公开接口。

当前 Android 构建配置声明最低 SDK 29、目标 SDK 35，构建目标为 `aarch64-linux-android` 和 `x86_64-linux-android`。应用不只面向 GrapheneOS。GrapheneOS、原生 Android、Samsung One UI、OnePlus OxygenOS 和其他 OEM 系统的验证状态分别记录，见 [兼容性状态](#兼容性状态)。

界面只参考公开录屏中的信息层级和交互关系。项目不使用华为代码、字体、图片、商标或专有服务。

## 当前已实现范围

当前版本包含以下功能：

- 天气首页和当前天气。
- 逐小时预报和五日预报。
- AQI 摘要和污染物状态。当前适配器可靠提供 PM2.5、PM10 的 AQI 子指数，其他污染物在数据缺失时显示不可用；监测站、趋势和历史属于当前版本排除范围。
- 降水雷达地图和时间轴。卫星、闪电、台风、灾害、温度和风力适配器仍在源码中，但属于当前版本排除范围；没有可靠帧时显示不可用，不伪造地图画面。
- 预警列表、地点和全国范围过滤、有效期、官方原文和复制操作。
- 城市搜索和单地点选择。
- 中文和 English 切换。
- 来源、更新时间、数据质量、加载、缓存、错误和不可用状态。
- Blender 5.3 原创云天空原型，以及桌面和 Android 可用的 egui 静态资源回退。

核心数据模型和适配器见 [`core/src/lib.rs`](core/src/lib.rs) 和 [`core/src/emb.rs`](core/src/emb.rs)。阶段报告见：

- [阶段 1：华为天气视频观察报告](docs/huawei-weather-video-observation-report.md)
- [阶段 2：项目盘点](docs/panahon-zh-project-inventory-stage2.md)
- [阶段 3：范围矩阵](docs/panahon-zh-scope-matrix-stage3.md)
- [阶段 4：前端和 Blender 报告](docs/panahon-zh-frontend-blender-stage4.md)

## 明确不属于当前版本范围

以下功能已明确排除，不作为当前版本的待实现任务：

- 15 日、90 日和长期预报。
- 霞光地图、月相和完整健康指数。
- 桌面小组件。
- Huawei 账号、云空间、钱包和系统推送。
- 专有天气视频、空间感天气资源和天气助手。
- Huawei 服务条款、隐私正文和专有资源。

阶段 3 的 B、C、D 组都按这个决定处理。若以后重新纳入，必须重新确认数据源、平台能力和测试条件。

## 数据来源和离线行为

| 功能 | 来源 | 说明 |
| --- | --- | --- |
| 实时天气、逐小时、五日预报 | PAGASA PANaHON | JSON 接口；显示来源、更新时间和质量 |
| 日出、日落、紫外线、能见度 | Open-Meteo | 位置模型预报 |
| 空气质量 | DENR-EMB | Power BI 公开报表；读不到时回退到 CAMS |
| AQI 趋势 | Open-Meteo / CAMS | 适配器仍保留；当前版本不纳入页面范围 |
| 降水雷达 | PAGASA | 当前版本只纳入降水时间帧；其他地图适配器保留但排除 |
| 预警 | PAGASA | 保留官方原文，并按当前地点和全国范围过滤 |

网络请求在后台线程执行。刷新失败时保留最近一次可信缓存，并显示失败原因。没有缓存时显示空状态。内置样例只在 `PANAHON_DEMO=1` 时使用。Android 另外保存地点、语言和最近的 AQI 历史记录。

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

- **GrapheneOS**：历史本机记录来自 Pixel 10a，记录时间为 2026-11-08，测试范围包括安装、启动、页面导航、天气和空气质量刷新、雷达图层、断网启动和地点搜索。当前云端没有重新连接该设备。
- **原生 Android**：API 和架构配置已声明，当前云端没有真机验证。
- **Samsung One UI、OnePlus OxygenOS 和其他 OEM 系统**：没有本项目的真机验证记录。

声明支持只表示构建配置覆盖该平台，不表示已经在所有设备上测试。详细历史记录见 [`docs/build-notes.md`](docs/build-notes.md)。

## 字体与布局限制

静态界面文案已经使用 Noto Sans CJK SC 重新生成字体子集，输出为 `assets/NotoSansCJKsc-Subset.otf`。动态城市名、预警正文和官方数据由网络返回，可能包含字体子集未覆盖的字符。需要保留 CJK fallback，或在发布前扩大字体子集。

当前没有完成所有分辨率、字体缩放、窄屏和 OEM 真机截图验证。因此不能保证所有设备都不会出现方框、文字截断或布局溢出。字体脚本见 [`tools/subset-font.py`](tools/subset-font.py)。

## 字体子集

从完整 CJK 字体导出字体面后运行：

```bash
python tools/subset-font.py --source /path/to/NotoSansCJKsc-Regular.otf
```

脚本扫描 `src/` 和 `core/src/`，生成 `assets/NotoSansCJKsc-Subset.otf`。动态数据字符不在扫描范围内，见上面的限制。

## 测试

工作区测试和桌面构建：

```bash
cargo test --locked --workspace --features network
cargo build --locked --features "desktop network"
```

Node 服务语法检查：

```bash
for file in services/emb-aqi/server.mjs services/emb-aqi/src/*.mjs services/emb-aqi/tools/*.mjs; do
  node --check "$file"
done
```

联网冒烟测试：

```bash
cargo run -p panahon-core --example live --features network
```

联网冒烟测试需要访问 PAGASA 和 Open-Meteo。网络受限时应记录原始连接错误，不要把它写成适配器通过或 Rust 代码失败。

## 代码和资源

```text
src/                         Rust/egui 页面、主题、图标和 Android 入口
core/                        数据模型、适配器、信任规则和联网示例
services/emb-aqi/            EMB AQI Node 服务
assets/blender_stage4/       Blender 5.3 场景和预览帧
tools/blender_stage4_cloud_sky.py
                             Blender 原型生成脚本
tools/subset-font.py         CJK 字体子集脚本
```

## 许可

代码使用 MIT 或 Apache-2.0 双许可证。字体使用 Noto Sans CJK SC，遵循 SIL Open Font License 1.1，许可证见 `assets/OFL.txt`。PAGASA、PANaHON、DENR-EMB 及其他名称、图像和商标不属于本项目。接入或再分发数据前，请确认各自的使用条款。
