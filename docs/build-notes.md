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

GrapheneOS 验证还需要一台开启 USB 调试的设备。当前没有连接设备，因此尚未执行 `adb install`、启动和权限测试。连接设备后运行 `adb devices`，再安装 APK。


