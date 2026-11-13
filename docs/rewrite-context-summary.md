# Rewrite context summary

## 当前目标

`panahon-zh` 是 Rust/egui Android 天气应用。界面只复刻公开录屏中的信息层级和交互关系。华为天气 APK 只作为 clean-room 观察证据，不复制其代码、字体、图片、着色器、商标或专有服务。

工作目录：`/workspace/panahon-zh`。

## 当前实现

阶段 3 A 组已经实现：

- 天气首页、当前天气、逐小时预报和五日预报。
- AQI 摘要、PM2.5/PM10 AQI 子指数和其他污染物的不可用状态；站点、趋势和历史代码保留，但属于当前版本排除范围。
- PAGASA 降水雷达和时间轴；卫星、闪电、台风和灾害适配器代码保留，但属于当前版本排除范围。
- 预警地点/全国范围过滤、有效期、官方原文和复制操作。
- 城市搜索、单地点选择、中文/English 切换。
- 来源、更新时间、`DataQuality`、缓存、加载、错误和不可用状态。
- 首页、AQI、地图、预警和设置之间的导航。
- 通过 `Option<f32>` 区分缺少降水和真实 `0 mm`；保留官方、预报、估算和不可用热指数质量；未知图标使用专用表现；生活指数需要完整逐小时和逐日输入。
- Blender 5.3 原创云天空场景、两张 PNG 预览帧和 egui 静态资源/光晕回退。

核心数据和适配器仍在 `core/src/lib.rs`、`core/src/emb.rs`、`services/emb-aqi/` 和地图适配器中。Android 缓存和 SharedPreferences 数据契约继续保留。

## 阶段 3 B/C/D

B、C、D 组当前都明确属于产品范围之外，不作为下一步任务。包括长期预报、霞光地图、月相、完整健康指数、温度/风力地图适配器、桌面小组件、系统级后台通知、专有天气视频、天气助手、资源下载、Huawei 账号/云空间/钱包/系统推送、服务条款和专有资源。

如果以后重新纳入，必须重新确认数据源、平台能力、许可、离线行为和测试条件。

## 当前验证状态

阶段 4 在云端执行了以下验证：

- `cargo test --locked --workspace --features network`：34 项通过。
- `cargo build --locked --features "desktop network"`：通过。
- Node `v27.0.0-alpha.0.1.0` 下的 `node --check`：通过。
- Python `3.15.2` 执行字体子集脚本：通过。
- Blender `5.3.0` 后台生成 `.blend` 和两张 PNG 预览帧：通过。
- `cargo run -p panahon-core --example live --features network`：程序能启动，但 PAGASA 返回 `Connection refused`，Open-Meteo 返回 `Network is unreachable`，所以当前云端没有完成联网数据验证。

当前云端没有 `DISPLAY` 或 `WAYLAND_DISPLAY`。桌面点击、全分辨率布局、Android 真机、系统剪贴板、网络地图帧、EMB Power BI 在线字段、通知权限和缓存恢复仍未验证。

## 历史 Android 验证

`docs/build-notes.md` 记录过 Pixel 10a 上的 GrapheneOS 验证。记录来自 2026-11-08 的历史提交，范围包括安装、启动、页面导航、天气和空气质量刷新、雷达图层、断网启动和地点搜索。

这条历史记录不代表当前云端重新验证，也不代表原生 Android、Samsung One UI、OnePlus OxygenOS 或其他 OEM 手机已经验证。当前构建配置声明 API 29+，目标架构为 `aarch64` 和 `x86_64`；32 位 ARM-only 设备不在目标范围内。

## 发布状态

当前仓库没有 GitHub Release，也没有可下载的官方 APK。APK 需要本地 Android SDK、NDK、`cargo-apk`、release keystore 和密码。APK、签名文件和密码不进入 Git 仓库。

## 字体与布局限制

静态文案已重新生成 CJK 字体子集。动态城市名、预警正文和官方数据可能包含子集未覆盖的字符，因此发布时必须保留 CJK fallback 或扩大子集。当前没有完成所有分辨率、字体缩放和 OEM 真机截图验证，不能保证所有设备没有方框或布局溢出。
