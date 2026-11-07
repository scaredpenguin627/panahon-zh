# PANaHON 中文版（Rust）

这是一个面向 GrapheneOS 的中文天气应用原型。项目使用 Rust 和 egui 编写，打包为普通 Android APK。用户关闭网络权限后，应用仍能显示离线数据。

项目参考了 PANaHON 1.1.4 XAPK 的公开功能。原应用是 Flutter 发布包，没有附带源码。我们只记录包信息、组件和用户可见功能，再独立实现数据模型和界面。项目不复制原包的 Flutter 二进制、商标、图片、文案、认证流程或私有令牌。

## 分析流程

第一步使用 IDA Pro 9.5 和 Hex-Rays IDA MCP 分析 APK 与 arm64 原生库。分析结果保存在 `docs/craft-anything-evidence/`。第二步使用 Craft Anything 保存证据，并根据证据编写 Rust 工程。Craft Anything 用于整理和复现分析结果，不替代 IDA Pro 的第一步。

## 功能

- 地点：搜索并切换常用地点。
- 预报：显示当前天气和五日预报。
- 地图：预留 PAGASA 雷达和观测图层。
- 预警：显示公开服务返回的天气预警。
- 离线模式：没有网络时使用内置样例数据。
- 网络模式：启用 `network` feature 后读取公开版本、观测、nowcast 和预警接口。

网络适配器不包含任何令牌。接口字段发生变化或请求失败时，应用会继续显示当前数据。

## 构建

主机需要以下工具：

- Rust 1.80 或更高版本。
- Android SDK、NDK 和 JDK 17 或更高版本。
- `cargo-apk`。

在 PowerShell 中执行：

```powershell
cargo install cargo-apk
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\<版本>"
cargo apk build --release --target aarch64-linux-android
```

APK 位于 `target\android-artifacts\release\`。启用公开接口适配器：

```powershell
cargo apk build --release --features network --target aarch64-linux-android
```

没有 Android SDK 时，可以先检查 Rust 代码和 Android 目标：

```powershell
cargo check --target aarch64-linux-android --workspace
```

## 在 GrapheneOS 上验证

使用自己的签名密钥安装 APK。不要复用原应用的签名。

```powershell
adb install -r <apk>
adb shell monkey -p org.panahon.zh -c android.intent.category.LAUNCHER 1
```

按下面的顺序检查：

1. 关闭网络权限，确认地点、预报和预警页面仍能打开。
2. 打开网络权限，点击“刷新”，确认请求失败时应用不会退出。
3. 只在选择定位功能时授予位置权限。
4. 在 Android 13 或更高版本上，确认通知权限由用户主动授予。
5. 使用 arm64-v8a APK 在 Pixel 或其他 GrapheneOS 设备上测试。

## 目录说明

- `src/`：egui 界面和 Android 入口。
- `core/`：数据模型、公开接口适配器和单元测试。
- `docs/evidence.md`：清洁室重建的证据边界和已知限制。
- `docs/ida-first-pass.md`：IDA Pro 第一轮分析和命令。
- `docs/craft-anything-evidence/`：Craft Anything 的分析说明。原始 JSON 只在本地生成，默认不纳入版本库。
- `build.ps1`：Android APK 构建脚本。
- `install-ida-mcp.ps1`：安装 IDA MCP 用户插件的脚本。

## 许可证和限制

代码使用 MIT 或 Apache-2.0 双许可证。PAGASA、PANaHON 和原应用中的其他名称、图像及商标不属于本项目。

项目目前没有完成真实 GrapheneOS 设备测试，也没有实现本地通知、雷达图像、观测历史和桌面小组件。接入更多公开数据前，请确认服务条款、速率限制和字段格式。


