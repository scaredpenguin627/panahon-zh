# 构建说明

当前文件记录本机的构建条件。完整打包还需要 `cargo-apk`、Android SDK、Android NDK 和 platform-tools。

Rust workspace 已通过 Windows 宿主检查。`aarch64-linux-android` 目标也已通过 `cargo check --target aarch64-linux-android --workspace`。类型检查不等于 APK 打包，生成 APK 时仍需配置 Android SDK 和 NDK。

设置环境变量后，在项目根目录运行 `build.ps1`：

```powershell
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\<版本>"
.\build.ps1
```

脚本默认生成 arm64 APK。需要网络接口适配器时，运行 `.\build.ps1 -Network`。

GrapheneOS 验证需要一台开启 USB 调试的设备。先安装 platform-tools，再运行 `adb devices` 确认设备已连接。



