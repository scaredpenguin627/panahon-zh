# 构建说明

本机已安装 Android SDK、NDK、platform-tools、JDK 27 和 `cargo-apk`。SDK 位于 `D:\IDA PRO 9.5\android-sdk`，版本为 API 35、build-tools 35.0.1、NDK 27.2.12479018。

Rust workspace 已通过 Windows 宿主检查，`aarch64-linux-android` 目标也已通过 `cargo check --target aarch64-linux-android --workspace --features network`。

## 宿主构建

宿主机没有 MSVC 链接器，桌面预览用 GNU 工具链：

```powershell
$env:PATH = "C:\msys64\ucrt64\bin;$env:PATH"
cargo +stable-x86_64-pc-windows-gnu run --features "desktop network"
```

编译 Android 目标时 `clang` 必须能搜到，否则 `ring` 会报 `failed to find tool "clang.exe"`：

```powershell
$env:PATH = "$env:ANDROID_NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin;$env:PATH"
```

## APK

```powershell
$env:CARGO_APK_RELEASE_KEYSTORE = "keystore\panahon-release.jks"
$env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = "<密码>"
.\build.ps1 -Network
```

包名 `org.panahon.zh`，最低 SDK 29，目标 SDK 35。`apksigner` 已通过 v3 签名检查，`zipalign` 已通过对齐检查。

同一台设备换签名后必须先卸载：

```powershell
adb uninstall org.panahon.zh
adb install -r target\release\apk\panahon-zh.apk
```

否则会报 `INSTALL_FAILED_UPDATE_INCOMPATIBLE`。

## 字体子集

界面文案改动后必须重新生成，否则新字会显示成方框：

```powershell
python tools\subset-font.py --source <NotoSansCJKsc-Regular.otf>
```

生成结果约 230 KB，覆盖 600 个字符左右。

## 截图

```powershell
pwsh -File tools\screenshot.ps1 -ProcessName panahon-zh -Out shot.png
```

脚本会先调用 `SetProcessDPIAware`。本机是 125% 缩放，不声明 DPI 感知时 `GetWindowRect` 返回虚拟化坐标，截出来的图只有窗口左上角一块。

真机截图直接用 `adb exec-out screencap -p`。设备息屏时截到的是全黑，先 `adb shell input keyevent KEYCODE_WAKEUP`。

## 验证记录

在 Pixel 10a（GrapheneOS）上完成：

- 安装、启动、底部导航五个页面切换。
- 联网刷新天气与空气质量，数据来自 PAGASA 与 DENR-EMB。
- 雷达图层加载最新帧，显示 9 帧时间轴和观测时间。
- 关闭 Wi-Fi 与移动数据后启动，显示内置样例，无崩溃。
- 页面滚动、预警列表、地点搜索框、通知按钮均可用。
