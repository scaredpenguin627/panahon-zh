# 构建说明

本文件区分历史 Android 验证和当前云端验证。当前云端工作目录是 `/workspace/panahon-zh`，分支是 `main`。

## 当前工具版本

阶段 4 云端记录的版本如下：

| 工具 | 版本 | 用途 |
| --- | --- | --- |
| Rust | `rustc 1.100.0`、`cargo 1.100.0` | workspace 测试和桌面构建 |
| Node | `v27.0.0-alpha.0.1.0` | EMB Node 服务语法检查 |
| Python | `3.15.2` | CJK 字体子集脚本 |
| Blender | `5.3.0` | 云天空原创动效原型 |

Python 3.15.2 的构建缺少若干可选模块，例如 `_sqlite3` 和 `_tkinter`。字体脚本不依赖这些模块。Blender 使用后台模式渲染，没有把系统 Blender 4.x 当作 5.3 使用。

## 当前云端验证

以下命令已在阶段 4 运行：

```bash
cargo test --locked --workspace --features network
cargo build --locked --features "desktop network"
for file in services/emb-aqi/server.mjs services/emb-aqi/src/*.mjs services/emb-aqi/tools/*.mjs; do
  node --check "$file"
done
```

Rust 测试通过 34 项，桌面构建通过，Node 语法检查通过。联网示例使用：

```bash
cargo run -p panahon-core --example live --features network
```

示例可以启动，但当前网络访问被拒绝：PAGASA 返回 `Connection refused`，Open-Meteo 返回 `Network is unreachable`。这次运行不能证明在线适配器已经验证。

Blender 原型使用：

```bash
/workspace/blender-5.3/blender-5.3.0-linux-x64/blender \
  --background --python tools/blender_stage4_cloud_sky.py
```

Blender 5.3.0 成功生成 `assets/blender_stage4/cloud_sky_scene.blend` 和两张 PNG 预览帧。没有 `DISPLAY` 或 `WAYLAND_DISPLAY`，所以没有完成桌面点击和视觉截图验证。

## Android APK 构建

Android 构建需要：

- Android SDK，建议 API 35 和对应 build-tools。
- Android NDK。`build.ps1` 默认查找 `27.2.12479018`。
- JDK 和 `cargo-apk`。安装：`cargo install cargo-apk`。
- release keystore 和 keystore 密码。

设置环境变量：

```powershell
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\27.2.12479018"
$env:CARGO_APK_RELEASE_KEYSTORE = "keystore\panahon-release.jks"
$env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = "<密码>"
```

构建两个 Android 目标：

```powershell
.\build.ps1 -Network -Target aarch64-linux-android
.\build.ps1 -Network -Target x86_64-linux-android
```

脚本会调用 `cargo apk build --release --lib`，设置 NDK clang，并加入 16 KB 页面大小链接参数。成功后的本地文件是 `target/release/apk/panahon-zh.apk`。普通 `cargo build` 不生成 APK。

签名文件、密码和 APK 不提交到 Git 仓库。`.gitignore` 会忽略 APK 和常见 keystore 文件。当前云端没有 Android SDK、NDK、cargo-apk 和签名材料的完整组合，因此本阶段没有重新生成或安装 APK。

## 历史 GrapheneOS 验证

历史记录来自对应历史提交及当时的构建说明：

- 2026-11-07 的历史记录包含 Android release 构建和 ELF 对齐检查。
- 2026-11-08 的历史记录包含 Pixel 10a（GrapheneOS）上的安装、启动、五页导航、天气和空气质量刷新、雷达图层、断网启动、页面滚动、预警列表、地点搜索和通知按钮测试。

这些记录是本机历史验证，不是当前云端重测。它们只覆盖 Pixel 10a 上的 GrapheneOS，不代表原生 Android、Samsung One UI、OnePlus OxygenOS 或其他 OEM 手机已验证。当前构建配置声明最低 API 29、目标 API 35，目标架构为 `aarch64` 和 `x86_64`；32 位 ARM-only 设备不属于当前目标。

## 字体子集

阶段 4 使用 Python 3.15.2 和 fontTools，从系统 CJK 字体集合导出字体面，再运行：

```bash
python tools/subset-font.py --source /workspace/toolchains/NotoSansCJK-Regular.otf
```

输出是 `assets/NotoSansCJKsc-Subset.otf`。脚本只扫描静态代码文案，动态城市名、预警正文和官方数据仍可能需要 CJK fallback 或更大的子集。

## 发布状态

当前仓库没有 GitHub Release，也没有官方可下载 APK。发布 APK 需要单独的签名、产物检查和上传流程。签名文件、密码和 APK 不进入 Git 仓库。

## 相关文档

- [阶段 1：华为天气视频观察报告](huawei-weather-video-observation-report.md)
- [阶段 2：项目盘点](panahon-zh-project-inventory-stage2.md)
- [阶段 3：范围矩阵](panahon-zh-scope-matrix-stage3.md)
- [阶段 4：前端和 Blender 报告](panahon-zh-frontend-blender-stage4.md)
- [构建脚本](../build.ps1)
