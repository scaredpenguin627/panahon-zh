# 构建说明

本机已安装 Android SDK、NDK、platform-tools 和 `cargo-apk`。SDK 位于本机工具目录，版本为 API 35、build-tools 35.0.1、NDK 27.2.12479018。

Rust workspace 已通过 Windows 宿主检查。`aarch64-linux-android` 目标也已通过 `cargo check --target aarch64-linux-android --workspace`。

本机已生成 `target\release\apk\panahon-zh.apk`。APK 包名为 `org.panahon.zh`，最低 SDK 为 29，目标 SDK 为 35。`apksigner` 已通过 v3 签名检查，`zipalign` 已通过对齐检查。

设置环境变量后，在项目根目录运行 `build.ps1`：

```powershell
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\<版本>"
.\build.ps1
```

脚本默认生成 arm64 APK。需要网络接口适配器时，运行 `.\build.ps1 -Network`。

GrapheneOS 验证已在 Pixel 10a 上完成。测试包括安装、启动、Network 权限、实时天气刷新、地点搜索、AQI 服务页和 Android 本地通知。远程雷达图像和后台定时通知仍需后续接入。



